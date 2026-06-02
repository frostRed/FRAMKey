const providerState = document.querySelector("#provider-state");
const providerSummary = document.querySelector("#provider-summary");
const output = document.querySelector("#dapp-output");
const eventLog = document.querySelector("#event-log");

const buttons = {
  chain: document.querySelector("#chain"),
  netVersion: document.querySelector("#net-version"),
  blockNumber: document.querySelector("#block-number"),
  accounts: document.querySelector("#accounts"),
  requestAccounts: document.querySelector("#request-accounts"),
  getPermissions: document.querySelector("#get-permissions"),
  requestPermissions: document.querySelector("#request-permissions"),
  revokePermissions: document.querySelector("#revoke-permissions"),
  addBaseChain: document.querySelector("#add-base-chain"),
  balance: document.querySelector("#balance"),
  watchAsset: document.querySelector("#watch-asset"),
  personalSign: document.querySelector("#personal-sign"),
  typedData: document.querySelector("#typed-data"),
  sendTransaction: document.querySelector("#send-transaction"),
};

let lastAccounts = [];
let wiredProvider = null;
let autosmokeStarted = false;

function provider() {
  return window.framkey ?? window.ethereum;
}

async function request(method, params = []) {
  const currentProvider = provider();
  if (!currentProvider?.request) {
    throw new Error("FRAMKey provider is unavailable");
  }
  setState(method, "busy");
  try {
    const result = await currentProvider.request({ method, params });
    if ((method === "eth_accounts" || method === "eth_requestAccounts") && Array.isArray(result)) {
      lastAccounts = result;
    }
    if (method === "wallet_revokePermissions") {
      lastAccounts = [];
    }
    render({ method, result });
    setState("Ready", "good");
    return { ok: true, result };
  } catch (error) {
    const rendered = {
      method,
      error: {
        code: error?.code,
        message: error?.message ?? String(error),
        data: error?.data,
      },
    };
    render(rendered);
    setState("Error", "bad");
    return { ok: false, error: rendered.error };
  }
}

function render(value) {
  output.textContent = JSON.stringify(value, null, 2);
}

function setState(text, tone) {
  providerState.textContent = text;
  providerState.dataset.tone = tone;
}

function refreshProviderState() {
  const currentProvider = provider();
  if (currentProvider?.request) {
    wireProviderEvents(currentProvider);
    const connected =
      typeof currentProvider.isConnected === "function" ? currentProvider.isConnected() : null;
    const chain = currentProvider.chainId ? ` chain ${currentProvider.chainId}` : "";
    const account = currentProvider.selectedAddress ? ` ${currentProvider.selectedAddress}` : "";
    const state = connected === null ? "" : connected ? " connected" : " disconnected";
    providerSummary.textContent = `${
      currentProvider.isFramKey ? "FRAMKey provider" : "Ethereum provider"
    }${state}${chain}${account}`;
    setState("Ready", "good");
  } else {
    providerSummary.textContent = "Provider pending";
    setState("Unavailable", "bad");
  }
}

function wireProviderEvents(currentProvider) {
  if (wiredProvider === currentProvider || typeof currentProvider.on !== "function") {
    return;
  }
  wiredProvider = currentProvider;
  currentProvider.on("connect", (value) => {
    appendEvent("connect", value);
    refreshProviderState();
  });
  currentProvider.on("accountsChanged", (accounts) => {
    lastAccounts = Array.isArray(accounts) ? accounts : [];
    appendEvent("accountsChanged", accounts);
    refreshProviderState();
  });
  currentProvider.on("chainChanged", (chainId) => {
    appendEvent("chainChanged", chainId);
    refreshProviderState();
  });
  currentProvider.on("disconnect", (error) => {
    appendEvent("disconnect", {
      code: error?.code,
      message: error?.message ?? String(error),
    });
    refreshProviderState();
  });
  currentProvider.on("message", (message) => appendEvent("message", message));
}

function appendEvent(type, value) {
  const item = document.createElement("li");
  const label = document.createElement("strong");
  const payload = document.createElement("span");
  label.textContent = type;
  payload.textContent = JSON.stringify(value ?? null);
  item.append(label, payload);
  eventLog.prepend(item);
  while (eventLog.children.length > 12) {
    eventLog.lastElementChild.remove();
  }
}

async function smokeReport(stage, detail = {}) {
  if (!window.__FRAMKEY_AUTOSMOKE__) {
    return;
  }
  const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return;
  }
  try {
    await invoke("framkey_smoke_event", { event: { stage, detail } });
  } catch {
    // Smoke reporting must not affect normal dApp behavior.
  }
}

function smokeSummary(method, response) {
  if (response?.ok) {
    const result = response.result;
    return {
      method,
      ok: true,
      resultKind: Array.isArray(result) ? "array" : typeof result,
      resultPreview:
        typeof result === "string"
          ? `${result.slice(0, 18)}${result.length > 18 ? "..." : ""}`
          : Array.isArray(result)
            ? `items=${result.length}`
            : null,
    };
  }
  return {
    method,
    ok: false,
    errorCode: response?.error?.code,
    errorMessage: response?.error?.message,
  };
}

async function smokeRequest(method, params = []) {
  const response = await request(method, params);
  await smokeReport("dapp_request", smokeSummary(method, response));
  return response;
}

async function startAutosmoke() {
  if (autosmokeStarted || !window.__FRAMKEY_AUTOSMOKE__) {
    return;
  }
  autosmokeStarted = true;
  await smokeReport("dapp_autosmoke_started", {
    origin: window.location.origin,
  });
  await smokeRequest("eth_chainId");
  await smokeRequest("eth_accounts");
  await smokeRequest("eth_requestAccounts");
  await smokeRequest("eth_accounts");
  await smokeRequest("wallet_watchAsset", {
    type: "ERC20",
    options: {
      address: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
      symbol: "USDC",
      decimals: 6,
      image: "https://static.alchemyapi.io/images/assets/3408.png",
    },
  });
  await smokeRequest("eth_signTypedData_v4", [
    lastAccounts[0] ?? "0x0000000000000000000000000000000000000000",
    {
      domain: {
        name: "USD Coin",
        version: "2",
        chainId: 1,
        verifyingContract: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
      },
      primaryType: "Permit",
      types: {
        EIP712Domain: [
          { name: "name", type: "string" },
          { name: "version", type: "string" },
          { name: "chainId", type: "uint256" },
          { name: "verifyingContract", type: "address" },
        ],
        Permit: [
          { name: "owner", type: "address" },
          { name: "spender", type: "address" },
          { name: "value", type: "uint256" },
          { name: "nonce", type: "uint256" },
          { name: "deadline", type: "uint256" },
        ],
      },
      message: {
        owner: lastAccounts[0] ?? "0x0000000000000000000000000000000000000000",
        spender: "0x000000000022d473030f116ddee9f6b43ac78ba3",
        value: "1000000",
        nonce: "0",
        deadline: "1900000000",
      },
    },
  ]);
  await smokeRequest("personal_sign", [
    "0x4652414d4b6579206175746f736d6f6b65",
    lastAccounts[0] ?? "0x0000000000000000000000000000000000000000",
  ]);
  await smokeRequest("eth_sendTransaction", [
    {
      from: lastAccounts[0] ?? "0x0000000000000000000000000000000000000000",
      to: "0x0000000000000000000000000000000000000001",
      value: "0x0",
      data:
        "0x095ea7b3" +
        "0000000000000000000000000000000000000000000000000000000000000002" +
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    },
  ]);
  await smokeReport("dapp_autosmoke_completed", {
    connectedAccounts: lastAccounts.length,
  });
}

buttons.chain.addEventListener("click", () => request("eth_chainId"));
buttons.netVersion.addEventListener("click", () => request("net_version"));
buttons.blockNumber.addEventListener("click", () => request("eth_blockNumber"));
buttons.accounts.addEventListener("click", () => request("eth_accounts"));
buttons.requestAccounts.addEventListener("click", () => request("eth_requestAccounts"));
buttons.getPermissions.addEventListener("click", () => request("wallet_getPermissions"));
buttons.requestPermissions.addEventListener("click", () =>
  request("wallet_requestPermissions", [{ eth_accounts: {} }]),
);
buttons.revokePermissions.addEventListener("click", () =>
  request("wallet_revokePermissions", [{ eth_accounts: {} }]),
);
buttons.addBaseChain.addEventListener("click", () =>
  request("wallet_addEthereumChain", [
    {
      chainId: "0x2105",
      chainName: "Base",
      nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
      rpcUrls: ["https://developer-provided-rpc.example/base"],
      blockExplorerUrls: ["https://basescan.org"],
    },
  ]),
);
buttons.balance.addEventListener("click", () =>
  request("eth_getBalance", [
    lastAccounts[0] ?? "0x0000000000000000000000000000000000000000",
    "latest",
  ]),
);
buttons.watchAsset.addEventListener("click", () =>
  request("wallet_watchAsset", {
    type: "ERC20",
    options: {
      address: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
      symbol: "USDC",
      decimals: 6,
      image: "https://static.alchemyapi.io/images/assets/3408.png",
    },
  }),
);
buttons.personalSign.addEventListener("click", () =>
  request("personal_sign", [
    "0x4652414d4b6579",
    lastAccounts[0] ?? "0x0000000000000000000000000000000000000000",
  ]),
);
buttons.typedData.addEventListener("click", () =>
  request("eth_signTypedData_v4", [
    lastAccounts[0] ?? "0x0000000000000000000000000000000000000000",
    {
      domain: {
        name: "USD Coin",
        version: "2",
        chainId: 1,
        verifyingContract: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
      },
      primaryType: "Permit",
      types: {
        EIP712Domain: [
          { name: "name", type: "string" },
          { name: "version", type: "string" },
          { name: "chainId", type: "uint256" },
          { name: "verifyingContract", type: "address" },
        ],
        Permit: [
          { name: "owner", type: "address" },
          { name: "spender", type: "address" },
          { name: "value", type: "uint256" },
          { name: "nonce", type: "uint256" },
          { name: "deadline", type: "uint256" },
        ],
      },
      message: {
        owner: lastAccounts[0] ?? "0x0000000000000000000000000000000000000000",
        spender: "0x000000000022d473030f116ddee9f6b43ac78ba3",
        value: "1000000",
        nonce: "0",
        deadline: "1900000000",
      },
    },
  ]),
);
buttons.sendTransaction.addEventListener("click", () =>
  request("eth_sendTransaction", [
    {
      from: lastAccounts[0] ?? "0x0000000000000000000000000000000000000000",
      to: "0x0000000000000000000000000000000000000001",
      value: "0x0",
      data:
        "0x095ea7b3" +
        "0000000000000000000000000000000000000000000000000000000000000002" +
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    },
  ]),
);

window.addEventListener("eip6963:announceProvider", (event) => {
  appendEvent("eip6963:announceProvider", event.detail?.info ?? null);
  refreshProviderState();
});
window.dispatchEvent(new Event("eip6963:requestProvider"));
refreshProviderState();
setTimeout(() => {
  startAutosmoke().catch((error) => {
    smokeReport("dapp_autosmoke_failed", { message: error?.message ?? String(error) });
  });
}, 400);
