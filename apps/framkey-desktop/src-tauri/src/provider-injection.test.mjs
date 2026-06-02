import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import vm from "node:vm";

const SCRIPT_PATH = new URL("./provider-injection.js", import.meta.url);
const script = await readFile(SCRIPT_PATH, "utf8");
const tests = [];

function createHarness({
  invoke,
  ethereum,
  remoteProviderSmoke,
  remoteProviderSmokeChainId,
  setTimeoutImpl,
} = {}) {
  const eventListeners = new Map();
  const dispatchedEvents = [];
  const harnessSetTimeout = setTimeoutImpl ?? setTimeout;
  const window = {
    location: { origin: "https://app.example", href: "https://app.example/swap?token=secret#frag" },
    __TAURI_INTERNALS__: {
      invoke:
        invoke ??
        (async (_command, args) => {
          const request = args.request;
          return { id: request.id, result: null };
        }),
    },
    setTimeout,
    addEventListener(type, listener) {
      const listeners = eventListeners.get(type) ?? new Set();
      listeners.add(listener);
      eventListeners.set(type, listeners);
    },
    removeEventListener(type, listener) {
      eventListeners.get(type)?.delete(listener);
    },
    dispatchEvent(event) {
      dispatchedEvents.push(event);
      for (const listener of Array.from(eventListeners.get(event.type) ?? [])) {
        listener.call(window, event);
      }
      return true;
    },
    setTimeout: harnessSetTimeout,
    clearTimeout,
  };
  if (ethereum !== undefined) {
    window.ethereum = ethereum;
  }
  if (remoteProviderSmoke !== undefined) {
    window.__FRAMKEY_REMOTE_PROVIDER_SMOKE__ = remoteProviderSmoke;
  }
  if (remoteProviderSmokeChainId !== undefined) {
    window.__FRAMKEY_REMOTE_PROVIDER_SMOKE_CHAIN_ID__ = remoteProviderSmokeChainId;
  }

  class CustomEvent {
    constructor(type, init = {}) {
      this.type = type;
      this.detail = init.detail;
    }
  }

  class Event {
    constructor(type) {
      this.type = type;
    }
  }

  const context = vm.createContext({
    window,
    CustomEvent,
    Event,
    setTimeout: harnessSetTimeout,
    clearTimeout,
    console,
  });
  vm.runInContext(script, context, { filename: "provider-injection.js" });
  return { window, dispatchedEvents };
}

function makeInvoke(handler) {
  const calls = [];
  const providerCalls = [];
  const telemetryCalls = [];
  const invoke = async (command, args) => {
    calls.push({ command, args });
    if (command === "framkey_provider_telemetry") {
      telemetryCalls.push({ command, args });
      return { id: "provider_telemetry", result: { recorded: true } };
    }
    assert.equal(command, "framkey_provider_request");
    providerCalls.push({ command, args });
    return handler(args.request);
  };
  invoke.calls = calls;
  invoke.providerCalls = providerCalls;
  invoke.telemetryCalls = telemetryCalls;
  return invoke;
}

function test(name, fn) {
  tests.push({ name, fn });
}

function plain(value) {
  return JSON.parse(JSON.stringify(value));
}

test("announces EIP-6963 provider without replacing an existing window.ethereum", () => {
  const existingEthereum = { existing: true };
  const { window, dispatchedEvents } = createHarness({ ethereum: existingEthereum });
  assert.equal(window.ethereum, existingEthereum);
  assert.equal(window.framkey.isFramKey, true);
  assert.equal(window.framkey.isMetaMask, false);

  const announcement = dispatchedEvents.find((event) => event.type === "eip6963:announceProvider");
  assert.ok(announcement);
  assert.equal(announcement.detail.provider, window.framkey);
  assert.equal(announcement.detail.info.name, "FRAMKey");
  assert.match(announcement.detail.info.icon, /^data:image\/png;base64,/);
  assert.equal(announcement.detail.info.icon.includes("svg"), false);
});

test("updates selectedAddress and emits accountsChanged after account requests", async () => {
  const invoke = makeInvoke((request) => {
    assert.equal(request.origin, "https://app.example");
    assert.equal(request.method, "eth_requestAccounts");
    return { id: request.id, result: ["0x1111111111111111111111111111111111111111"] };
  });
  const { window } = createHarness({ invoke });
  const events = [];
  window.framkey.on("accountsChanged", (accounts) => events.push(accounts));

  const accounts = await window.framkey.request({ method: "eth_requestAccounts" });

  assert.deepEqual(plain(accounts), ["0x1111111111111111111111111111111111111111"]);
  assert.equal(window.framkey.selectedAddress, "0x1111111111111111111111111111111111111111");
  assert.deepEqual(plain(events), [["0x1111111111111111111111111111111111111111"]]);
  assert.equal(invoke.providerCalls.length, 1);
});

test("updates chain state and emits connect and chainChanged", async () => {
  const invoke = makeInvoke((request) => {
    if (request.method === "eth_chainId") {
      return { id: request.id, result: "0x1" };
    }
    if (request.method === "wallet_switchEthereumChain") {
      return { id: request.id, result: null };
    }
    throw new Error(`unexpected method ${request.method}`);
  });
  const { window } = createHarness({ invoke });
  const connectEvents = [];
  const chainEvents = [];
  window.framkey.on("connect", (value) => connectEvents.push(value));
  window.framkey.on("chainChanged", (value) => chainEvents.push(value));

  await window.framkey.request({ method: "eth_chainId" });
  await window.framkey.request({
    method: "wallet_switchEthereumChain",
    params: [{ chainId: "0x5" }],
  });

  assert.equal(window.framkey.isConnected(), true);
  assert.equal(window.framkey.chainId, "0x5");
  assert.equal(window.framkey.networkVersion, "5");
  assert.deepEqual(plain(connectEvents), [{ chainId: "0x1" }]);
  assert.deepEqual(plain(chainEvents), ["0x5"]);
});

test("does not update chain state when switch is rejected", async () => {
  const invoke = makeInvoke((request) => {
    if (request.method === "eth_chainId") {
      return { id: request.id, result: "0x1" };
    }
    if (request.method === "wallet_switchEthereumChain") {
      return {
        id: request.id,
        error: {
          code: 4902,
          message: "unsupported chain",
          data: { requestedChainId: "0x5" },
        },
      };
    }
    throw new Error(`unexpected method ${request.method}`);
  });
  const { window } = createHarness({ invoke });
  const chainEvents = [];
  window.framkey.on("chainChanged", (value) => chainEvents.push(value));

  await window.framkey.request({ method: "eth_chainId" });
  await assert.rejects(
    () =>
      window.framkey.request({
        method: "wallet_switchEthereumChain",
        params: [{ chainId: "0x5" }],
      }),
    (error) => {
      assert.equal(error.code, 4902);
      assert.equal(error.message, "unsupported chain");
      return true;
    },
  );

  assert.equal(window.framkey.chainId, "0x1");
  assert.equal(window.framkey.networkVersion, "1");
  assert.deepEqual(chainEvents, []);
});

test("supports once, off, listenerCount, and wrapper removal", () => {
  const { window } = createHarness();
  let onceCount = 0;
  window.framkey.once("message", () => {
    onceCount += 1;
  });
  assert.equal(window.framkey.listenerCount("message"), 1);
  window.framkey.emit("message", { type: "first" });
  window.framkey.emit("message", { type: "second" });
  assert.equal(onceCount, 1);
  assert.equal(window.framkey.listenerCount("message"), 0);

  let removedCount = 0;
  const removed = () => {
    removedCount += 1;
  };
  window.framkey.once("chainChanged", removed);
  window.framkey.off("chainChanged", removed);
  window.framkey.emit("chainChanged", "0x2");
  assert.equal(removedCount, 0);
});

test("reports provider lifecycle telemetry without using provider requests", async () => {
  const invoke = makeInvoke((request) => {
    throw new Error(`unexpected provider request ${request.method}`);
  });
  const { window } = createHarness({ invoke });

  window.dispatchEvent(new Event("eip6963:requestProvider"));
  await new Promise((resolve) => setTimeout(resolve, 0));

  assert.equal(invoke.providerCalls.length, 0);
  assert.ok(
    invoke.telemetryCalls.some((call) => call.args.event.event === "provider_injected"),
  );
  assert.ok(invoke.telemetryCalls.every((call) => !call.args.event.url.includes("token=secret")));
  assert.ok(invoke.telemetryCalls.every((call) => !call.args.event.url.includes("#frag")));
  assert.ok(
    invoke.telemetryCalls.some(
      (call) => call.args.event.event === "eip6963_request_provider",
    ),
  );
  assert.ok(
    invoke.telemetryCalls.some(
      (call) => call.args.event.event === "eip6963_announce_provider",
    ),
  );
});

test("supports enable, send, and sendAsync compatibility methods", async () => {
  const invoke = makeInvoke((request) => {
    if (request.method === "eth_requestAccounts") {
      return { id: request.id, result: ["0x2222222222222222222222222222222222222222"] };
    }
    if (request.method === "eth_chainId") {
      return { id: request.id, result: "0x1" };
    }
    if (request.method === "net_version") {
      return { id: request.id, result: "1" };
    }
    throw new Error(`unexpected method ${request.method}`);
  });
  const { window } = createHarness({ invoke });

  assert.deepEqual(plain(await window.framkey.enable()), [
    "0x2222222222222222222222222222222222222222",
  ]);
  assert.equal(await window.framkey.send("eth_chainId"), "0x1");

  const sendResponse = await new Promise((resolve, reject) => {
    window.framkey.send({ jsonrpc: "2.0", id: 7, method: "net_version" }, (error, response) => {
      if (error) {
        reject(error);
      } else {
        resolve(response);
      }
    });
  });
  assert.deepEqual(plain(sendResponse), { jsonrpc: "2.0", id: 7, result: "1" });

  const batchResponse = await new Promise((resolve, reject) => {
    window.framkey.sendAsync(
      [
        { jsonrpc: "2.0", id: 8, method: "eth_chainId" },
        { jsonrpc: "2.0", id: 9, method: "net_version" },
      ],
      (error, response) => {
        if (error) {
          reject(error);
        } else {
          resolve(response);
        }
      },
    );
  });
  assert.deepEqual(plain(batchResponse), [
    { jsonrpc: "2.0", id: 8, result: "0x1" },
    { jsonrpc: "2.0", id: 9, result: "1" },
  ]);
});

test("clears selectedAddress and emits accountsChanged after permission revocation", async () => {
  const invoke = makeInvoke((request) => {
    if (request.method === "eth_requestAccounts") {
      return { id: request.id, result: ["0x3333333333333333333333333333333333333333"] };
    }
    if (request.method === "wallet_revokePermissions") {
      return { id: request.id, result: null };
    }
    throw new Error(`unexpected method ${request.method}`);
  });
  const { window } = createHarness({ invoke });
  const events = [];
  window.framkey.on("accountsChanged", (accounts) => events.push(accounts));

  await window.framkey.request({ method: "eth_requestAccounts" });
  await window.framkey.request({
    method: "wallet_revokePermissions",
    params: [{ eth_accounts: {} }],
  });

  assert.equal(window.framkey.selectedAddress, null);
  assert.deepEqual(plain(events), [["0x3333333333333333333333333333333333333333"], []]);
});

test("turns provider errors into FramKeyProviderRpcError instances", async () => {
  const invoke = makeInvoke((request) => ({
    id: request.id,
    error: {
      code: 4100,
      message: "account mismatch",
      data: { reason: "test" },
    },
  }));
  const { window } = createHarness({ invoke });

  await assert.rejects(
    () => window.framkey.request({ method: "eth_requestAccounts" }),
    (error) => {
      assert.equal(error.name, "FramKeyProviderRpcError");
      assert.equal(error.code, 4100);
      assert.equal(error.message, "account mismatch");
      assert.deepEqual(plain(error.data), { reason: "test" });
      return true;
    },
  );
});

test("interactive remote provider smoke connects, signs, and hides signature previews", async () => {
  const account = "0x4444444444444444444444444444444444444444";
  const signature = `0x${"a".repeat(130)}`;
  const typedSignature = `0x${"c".repeat(130)}`;
  const transactionHash = `0x${"b".repeat(64)}`;
  let connected = false;
  let chainId = "0x1";
  const invoke = makeInvoke((request) => {
    if (request.method === "eth_chainId") {
      return { id: request.id, result: chainId };
    }
    if (request.method === "eth_accounts") {
      return { id: request.id, result: connected ? [account] : [] };
    }
    if (request.method === "eth_blockNumber") {
      return { id: request.id, result: "0x123" };
    }
    if (request.method === "wallet_switchEthereumChain") {
      assert.deepEqual(plain(request.params), [{ chainId: "0x2105" }]);
      chainId = "0x2105";
      return { id: request.id, result: null };
    }
    if (request.method === "eth_requestAccounts") {
      connected = true;
      return { id: request.id, result: [account] };
    }
    if (request.method === "personal_sign") {
      assert.deepEqual(plain(request.params), [
        "0x4652414d4b65792072656d6f746520736d6f6b65",
        account,
      ]);
      return { id: request.id, result: signature };
    }
    if (request.method === "eth_signTypedData_v4") {
      assert.equal(request.params[0], account);
      assert.equal(request.params[1].primaryType, "PermitSingle");
      assert.equal(request.params[1].domain.chainId, "8453");
      assert.equal(
        request.params[1].domain.verifyingContract,
        "0x000000000022d473030f116ddee9f6b43ac78ba3",
      );
      assert.equal(
        request.params[1].message.details.token,
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
      );
      assert.equal(
        request.params[1].message.spender,
        "0x6ff5693b99212da76ad316178a184ab56d299b43",
      );
      return { id: request.id, result: typedSignature };
    }
    if (request.method === "eth_sendTransaction") {
      assert.deepEqual(plain(request.params), [
        {
          from: account,
          to: "0x0000000000000000000000000000000000000001",
          value: "0x0",
          data: "0x",
        },
      ]);
      return { id: request.id, result: transactionHash };
    }
    throw new Error(`unexpected method ${request.method}`);
  });
  createHarness({
    invoke,
    remoteProviderSmoke: "interactive",
    remoteProviderSmokeChainId: "0x2105",
    setTimeoutImpl: (fn, delay) => {
      if (delay >= 30_000) {
        return setTimeout(fn, delay);
      }
      Promise.resolve().then(fn);
      return 1;
    },
  });

  await new Promise((resolve) => setTimeout(resolve, 0));
  await new Promise((resolve) => setTimeout(resolve, 0));

  assert.deepEqual(
    invoke.providerCalls.map((call) => call.args.request.method),
    [
      "eth_chainId",
      "eth_accounts",
      "eth_blockNumber",
      "wallet_switchEthereumChain",
      "eth_chainId",
      "eth_blockNumber",
      "eth_requestAccounts",
      "eth_accounts",
      "personal_sign",
      "eth_signTypedData_v4",
      "eth_sendTransaction",
    ],
  );
  const smokeDetails = invoke.telemetryCalls
    .filter((call) => call.args.event.event === "provider_smoke_request")
    .map((call) => call.args.event.detail);
  assert.ok(
    smokeDetails.some(
      (detail) => detail.method === "personal_sign" && detail.resultPreview === "signature",
    ),
  );
  assert.ok(
    smokeDetails.some(
      (detail) =>
        detail.method === "eth_signTypedData_v4" && detail.resultPreview === "signature",
    ),
  );
  assert.ok(
    smokeDetails.some(
      (detail) =>
        detail.method === "eth_sendTransaction" && detail.resultPreview === "transaction_hash",
    ),
  );
  assert.equal(JSON.stringify(invoke.telemetryCalls).includes(signature.slice(2, 18)), false);
  assert.equal(JSON.stringify(invoke.telemetryCalls).includes(typedSignature.slice(2, 18)), false);
});

for (const { name, fn } of tests) {
  try {
    await fn();
    console.log(`ok - ${name}`);
  } catch (error) {
    console.error(`not ok - ${name}`);
    throw error;
  }
}
