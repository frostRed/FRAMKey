(function () {
  "use strict";

  const PROVIDER_ICON =
    "data:image/png;base64," +
    "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";

  const PROVIDER_INFO = {
    uuid: "b7b46ee4-48bc-4050-a02f-000000000001",
    name: "FRAMKey",
    icon: PROVIDER_ICON,
    rdns: "dev.framkey",
  };
  const PROVIDER_SMOKE_TIMEOUT_MS = 30_000;
  const PROVIDER_SMOKE_TX_TO = "0x0000000000000000000000000000000000000001";
  const PROVIDER_SMOKE_PERMIT_TOKEN = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
  const PROVIDER_SMOKE_PERMIT2_CONTRACT = "0x000000000022d473030f116ddee9f6b43ac78ba3";
  const PROVIDER_SMOKE_PERMIT_SPENDERS = {
    "0x1": "0x66a9893cc07d91d95644aedd05d03f95e1dba8af",
    "0xaa36a7": "0x3a9d48ab9751398bbfa63ad67599bb04e4bdf98b",
    "0x2105": "0x6ff5693b99212da76ad316178a184ab56d299b43",
    "0xa": "0x851116d9223fabed8e56c0e6b8ad0c31d98b3507",
    "0xa4b1": "0xa51afafe0263b40edaef0df8781ea9aa03e381a3",
    "0x89": "0x1095692a6237d83c6a72f3f5efedb9a670c49223",
  };

  class FramKeyProviderRpcError extends Error {
    constructor(error) {
      super(error?.message ?? "FRAMKey provider request failed");
      this.name = "FramKeyProviderRpcError";
      this.code = Number.isInteger(error?.code) ? error.code : 4200;
      this.data = error?.data;
    }
  }

  class FramKeyProvider {
    constructor() {
      this.isFramKey = true;
      this.isMetaMask = false;
      this.selectedAddress = null;
      this.chainId = null;
      this.networkVersion = null;
      this._accounts = [];
      this._connected = false;
      this._nextId = 1;
      this._listeners = new Map();
    }

    async request(args) {
      if (!args || typeof args.method !== "string" || args.method.length === 0) {
        throw new FramKeyProviderRpcError({
          code: -32602,
          message: "FRAMKey request requires a string method",
        });
      }

      const invoke = tauriInvoke();
      if (!invoke) {
        throw new FramKeyProviderRpcError({
          code: 4900,
          message: "FRAMKey Tauri bridge is unavailable",
        });
      }

      const id = `framkey_${this._nextId++}`;
      let response;
      try {
        response = await invoke("framkey_provider_request", {
          request: {
            id,
            method: args.method,
            params: args.params ?? [],
            origin: currentOrigin(),
          },
        });
      } catch (error) {
        throw new FramKeyProviderRpcError({
          code: 4900,
          message: error?.message ?? "FRAMKey provider bridge failed",
          data: serializeProviderError(error),
        });
      }

      if (response?.error) {
        throw new FramKeyProviderRpcError(response.error);
      }

      const result = response?.result;
      this._updateState(args.method, args.params ?? [], result);
      return result;
    }

    on(eventName, listener) {
      if (typeof listener !== "function") {
        return this;
      }
      const listeners = this._listeners.get(eventName) ?? new Set();
      listeners.add(listener);
      this._listeners.set(eventName, listeners);
      return this;
    }

    addListener(eventName, listener) {
      return this.on(eventName, listener);
    }

    removeListener(eventName, listener) {
      const listeners = this._listeners.get(eventName);
      if (!listeners) {
        return this;
      }
      for (const registered of Array.from(listeners)) {
        if (registered === listener || registered._framkeyOriginal === listener) {
          listeners.delete(registered);
        }
      }
      if (listeners.size === 0) {
        this._listeners.delete(eventName);
      }
      return this;
    }

    off(eventName, listener) {
      return this.removeListener(eventName, listener);
    }

    once(eventName, listener) {
      if (typeof listener !== "function") {
        return this;
      }
      const wrapper = (...args) => {
        this.removeListener(eventName, wrapper);
        listener(...args);
      };
      wrapper._framkeyOriginal = listener;
      return this.on(eventName, wrapper);
    }

    listenerCount(eventName) {
      return this._listeners.get(eventName)?.size ?? 0;
    }

    listeners(eventName) {
      return Array.from(this._listeners.get(eventName) ?? []);
    }

    emit(eventName, ...args) {
      const listeners = Array.from(this._listeners.get(eventName) ?? []);
      if (listeners.length === 0) {
        return false;
      }
      for (const listener of listeners) {
        try {
          listener(...args);
        } catch (error) {
          reportListenerError(error);
        }
      }
      return true;
    }

    isConnected() {
      return this._connected;
    }

    enable() {
      return this.request({ method: "eth_requestAccounts" });
    }

    send(methodOrPayload, paramsOrCallback) {
      if (typeof methodOrPayload === "string") {
        return this.request({
          method: methodOrPayload,
          params: paramsOrCallback ?? [],
        });
      }
      if (typeof paramsOrCallback === "function") {
        return this.sendAsync(methodOrPayload, paramsOrCallback);
      }
      if (methodOrPayload && typeof methodOrPayload === "object") {
        return this.request({
          method: methodOrPayload.method,
          params: methodOrPayload.params ?? [],
        });
      }
      return Promise.reject(
        new FramKeyProviderRpcError({
          code: -32602,
          message: "FRAMKey send requires a method or JSON-RPC payload",
        }),
      );
    }

    sendAsync(payload, callback) {
      if (typeof callback !== "function") {
        throw new TypeError("FRAMKey sendAsync requires a callback");
      }
      Promise.resolve()
        .then(() => {
          if (Array.isArray(payload)) {
            return Promise.all(payload.map((item) => this._jsonRpcResponse(item)));
          }
          return this._jsonRpcResponse(payload);
        })
        .then(
          (response) => callback(null, response),
          (error) => callback(error, null),
        );
      return undefined;
    }

    async _jsonRpcResponse(payload) {
      const id = payload && hasOwn(payload, "id") ? payload.id : null;
      const jsonrpc = payload?.jsonrpc ?? "2.0";
      if (!payload || typeof payload.method !== "string") {
        return {
          jsonrpc,
          id,
          error: {
            code: -32600,
            message: "Invalid JSON-RPC request",
          },
        };
      }
      try {
        const result = await this.request({
          method: payload.method,
          params: payload.params ?? [],
        });
        return { jsonrpc, id, result };
      } catch (error) {
        return {
          jsonrpc,
          id,
          error: serializeProviderError(error),
        };
      }
    }

    _updateState(method, params, result) {
      if (method === "eth_chainId") {
        this._updateChain(result);
        return;
      }
      if (method === "net_version" && typeof result === "string") {
        this.networkVersion = result;
        return;
      }
      if (method === "eth_accounts" || method === "eth_requestAccounts") {
        this._updateAccounts(normalizeAccounts(result));
        return;
      }
      if (method === "eth_coinbase" && typeof result === "string") {
        this._updateAccounts([result]);
        return;
      }
      if (method === "framkey_getAccount" && result && typeof result === "object") {
        if (typeof result.chainId === "string") {
          this._updateChain(result.chainId);
        }
        if (typeof result.address === "string") {
          this._updateAccounts([result.address]);
        }
        return;
      }
      if (method === "wallet_switchEthereumChain") {
        const requestedChainId = chainIdFromSwitchParams(params);
        if (requestedChainId) {
          this._updateChain(requestedChainId);
        }
        return;
      }
      if (method === "wallet_revokePermissions" && requestsEthAccountsPermission(params)) {
        this._updateAccounts([]);
      }
    }

    _updateAccounts(accounts) {
      if (!sameStringArray(this._accounts, accounts)) {
        this._accounts = accounts;
        this.selectedAddress = accounts[0] ?? null;
        this.emit("accountsChanged", [...accounts]);
      } else if (this.selectedAddress !== (accounts[0] ?? null)) {
        this.selectedAddress = accounts[0] ?? null;
      }
    }

    _updateChain(chainId) {
      const normalized = normalizeChainId(chainId);
      if (!normalized) {
        return;
      }
      const previous = this.chainId;
      this.chainId = normalized;
      this.networkVersion = decimalChainId(normalized);
      if (!this._connected) {
        this._connected = true;
        this.emit("connect", { chainId: normalized });
        return;
      }
      if (previous && previous.toLowerCase() !== normalized.toLowerCase()) {
        this.emit("chainChanged", normalized);
      }
    }
  }

  function tauriInvoke() {
    return window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
  }

  function recordTelemetry(event, detail = {}) {
    const invoke = tauriInvoke();
    if (!invoke) {
      return;
    }
    Promise.resolve(
      invoke("framkey_provider_telemetry", {
        event: {
          event,
          origin: currentOrigin(),
          url: currentUrl(),
          detail,
        },
      }),
    ).catch(() => {});
  }

  function currentOrigin() {
    try {
      return window.location?.origin ?? "null";
    } catch {
      return "null";
    }
  }

  function currentUrl() {
    try {
      const href = window.location?.href ?? "about:blank";
      const url = new URL(href);
      url.search = "";
      url.hash = "";
      return url.toString();
    } catch {
      return "about:blank";
    }
  }

  function normalizeAccounts(value) {
    if (!Array.isArray(value)) {
      return [];
    }
    return value.filter((item) => typeof item === "string");
  }

  function sameStringArray(left, right) {
    if (left.length !== right.length) {
      return false;
    }
    return left.every((value, index) => value === right[index]);
  }

  function normalizeChainId(value) {
    if (typeof value !== "string" || !/^0x[0-9a-fA-F]+$/.test(value)) {
      return null;
    }
    try {
      return `0x${BigInt(value).toString(16)}`;
    } catch {
      return value.toLowerCase();
    }
  }

  function decimalChainId(value) {
    try {
      return BigInt(value).toString(10);
    } catch {
      return null;
    }
  }

  function chainIdFromSwitchParams(params) {
    const first = Array.isArray(params) ? params[0] : null;
    return normalizeChainId(first?.chainId);
  }

  function requestsEthAccountsPermission(params) {
    const first = Array.isArray(params) ? params[0] : null;
    return Boolean(first && typeof first === "object" && hasOwn(first, "eth_accounts"));
  }

  function hasOwn(value, key) {
    return Object.prototype.hasOwnProperty.call(value, key);
  }

  function serializeProviderError(error) {
    const code = Number.isInteger(error?.code) ? error.code : 4900;
    return {
      code,
      message: error?.message ?? "FRAMKey provider request failed",
      data: error?.data,
    };
  }

  function reportListenerError(error) {
    const rethrow = () => {
      throw error;
    };
    if (typeof window.setTimeout === "function") {
      window.setTimeout(rethrow, 0);
    } else if (typeof setTimeout === "function") {
      setTimeout(rethrow, 0);
    }
  }

  function providerSmokeSummary(method, result) {
    let resultPreview = null;
    if (method === "personal_sign" || method === "eth_signTypedData_v4") {
      resultPreview = typeof result === "string" ? "signature" : null;
    } else if (method === "eth_sendTransaction") {
      resultPreview = typeof result === "string" ? "transaction_hash" : null;
    } else if (typeof result === "string") {
      resultPreview = `${result.slice(0, 18)}${result.length > 18 ? "..." : ""}`;
    } else if (Array.isArray(result)) {
      resultPreview = `items=${result.length}`;
    }
    return {
      method,
      ok: true,
      resultKind: Array.isArray(result) ? "array" : typeof result,
      resultPreview,
    };
  }

  function providerSmokeError(method, error) {
    return {
      method,
      ok: false,
      errorCode: Number.isInteger(error?.code) ? error.code : null,
      errorMessage:
        typeof error?.message === "string" ? error.message.slice(0, 160) : "provider smoke failed",
    };
  }

  function remoteProviderSmokeMode(options = {}) {
    const rawMode = options.mode ?? window.__FRAMKEY_REMOTE_PROVIDER_SMOKE__;
    if (!rawMode) {
      return null;
    }
    if (rawMode === true) {
      return "read";
    }
    const mode = String(rawMode).trim().toLowerCase();
    if (["0", "false", "no", "off"].includes(mode)) {
      return null;
    }
    if (["interactive", "full", "write", "sign"].includes(mode)) {
      return "interactive";
    }
    return "read";
  }

  function remoteProviderSmokeChainId(options = {}) {
    return normalizeChainId(options.chainId ?? window.__FRAMKEY_REMOTE_PROVIDER_SMOKE_CHAIN_ID__);
  }

  async function providerSmokeRequest(provider, method, params = []) {
    try {
      const result = await withProviderSmokeTimeout(provider.request({ method, params }), method);
      recordTelemetry("provider_smoke_request", providerSmokeSummary(method, result));
      return { ok: true, result };
    } catch (error) {
      recordTelemetry("provider_smoke_request", providerSmokeError(method, error));
      return { ok: false, error };
    }
  }

  async function withProviderSmokeTimeout(promise, method) {
    let timeoutId = null;
    const timeout = new Promise((_, reject) => {
      timeoutId = setTimeout(() => {
        reject(new Error(`provider smoke ${method} timed out`));
      }, PROVIDER_SMOKE_TIMEOUT_MS);
    });
    try {
      return await Promise.race([promise, timeout]);
    } finally {
      if (timeoutId !== null) {
        clearTimeout(timeoutId);
      }
    }
  }

  async function runRemoteProviderSmoke(provider, options = {}) {
    const mode = remoteProviderSmokeMode(options);
    if (!mode) {
      return;
    }
    recordTelemetry("provider_smoke_started", {
      provider: PROVIDER_INFO.rdns,
      mode,
      source: options.source ?? "startup",
    });
    for (const method of ["eth_chainId", "eth_accounts", "eth_blockNumber"]) {
      await providerSmokeRequest(provider, method);
    }
    if (mode === "interactive") {
      const targetChainId = remoteProviderSmokeChainId(options);
      if (targetChainId) {
        const switchRequest = await providerSmokeRequest(provider, "wallet_switchEthereumChain", [
          { chainId: targetChainId },
        ]);
        const switchedChain = await providerSmokeRequest(provider, "eth_chainId");
        if (!switchRequest.ok || !switchedChain.ok) {
          recordTelemetry("provider_smoke_skipped", {
            provider: PROVIDER_INFO.rdns,
            mode,
            reason: "chain_switch_failed",
            targetChainId,
          });
          recordTelemetry("provider_smoke_completed", {
            provider: PROVIDER_INFO.rdns,
            mode,
          });
          return;
        }
        const observedChainId = normalizeChainId(switchedChain.result);
        if (observedChainId !== targetChainId) {
          recordTelemetry("provider_smoke_skipped", {
            provider: PROVIDER_INFO.rdns,
            mode,
            reason: "chain_switch_mismatch",
            targetChainId,
            observedChainId,
          });
          recordTelemetry("provider_smoke_completed", {
            provider: PROVIDER_INFO.rdns,
            mode,
          });
          return;
        }
        await providerSmokeRequest(provider, "eth_blockNumber");
      }
      const accountRequest = await providerSmokeRequest(provider, "eth_requestAccounts");
      const connectedAccounts = accountRequest.ok ? normalizeAccounts(accountRequest.result) : [];
      await providerSmokeRequest(provider, "eth_accounts");
      const account = connectedAccounts[0] ?? provider.selectedAddress;
      if (account) {
        await providerSmokeRequest(provider, "personal_sign", [
          providerSmokeSiweMessage(account, provider.chainId),
          account,
        ]);
        await providerSmokeRequest(provider, "eth_signTypedData_v4", [
          account,
          providerSmokePermitTypedData(provider.chainId),
        ]);
        await providerSmokeRequest(provider, "eth_sendTransaction", [
          {
            from: account,
            to: PROVIDER_SMOKE_TX_TO,
            value: "0x0",
            data: "0x",
          },
        ]);
      } else {
        recordTelemetry("provider_smoke_skipped", {
          provider: PROVIDER_INFO.rdns,
          mode,
          reason: "no_connected_account",
        });
      }
    }
    recordTelemetry("provider_smoke_completed", {
      provider: PROVIDER_INFO.rdns,
      mode,
    });
  }

  function providerSmokePermitTypedData(chainIdHex) {
    const chainId = decimalChainId(chainIdHex) ?? "1";
    const normalizedChainIdHex = normalizeChainIdHex(chainIdHex) ?? "0x1";
    const deadline = String(Math.floor(Date.now() / 1000) + 3600);
    return {
      domain: {
        name: "Permit2",
        chainId,
        verifyingContract: PROVIDER_SMOKE_PERMIT2_CONTRACT,
      },
      primaryType: "PermitSingle",
      types: {
        EIP712Domain: [
          { name: "name", type: "string" },
          { name: "chainId", type: "uint256" },
          { name: "verifyingContract", type: "address" },
        ],
        PermitDetails: [
          { name: "token", type: "address" },
          { name: "amount", type: "uint160" },
          { name: "expiration", type: "uint48" },
          { name: "nonce", type: "uint48" },
        ],
        PermitSingle: [
          { name: "details", type: "PermitDetails" },
          { name: "spender", type: "address" },
          { name: "sigDeadline", type: "uint256" },
        ],
      },
      message: {
        details: {
          token: PROVIDER_SMOKE_PERMIT_TOKEN,
          amount: "1000000",
          expiration: deadline,
          nonce: "0",
        },
        spender:
          PROVIDER_SMOKE_PERMIT_SPENDERS[normalizedChainIdHex] ??
          PROVIDER_SMOKE_PERMIT_SPENDERS["0x1"],
        sigDeadline: deadline,
      },
    };
  }

  function providerSmokeSiweMessage(account, chainIdHex) {
    const origin = providerSmokeOrigin();
    const domain = providerSmokeAuthority(origin);
    const chainId = decimalChainId(chainIdHex) ?? "1";
    const issuedAt = providerSmokeTimestamp(Date.now());
    const expirationTime = providerSmokeTimestamp(Date.now() + 5 * 60_000);
    const nonce = `FRAMKey${Math.floor(Date.now() / 1000).toString(36)}`;
    return [
      `${domain} wants you to sign in with your Ethereum account:`,
      account,
      "",
      "FRAMKey remote smoke",
      "",
      `URI: ${origin}`,
      "Version: 1",
      `Chain ID: ${chainId}`,
      `Nonce: ${nonce}`,
      `Issued At: ${issuedAt}`,
      `Expiration Time: ${expirationTime}`,
    ].join("\n");
  }

  function providerSmokeTimestamp(ms) {
    return new Date(ms).toISOString().replace(/\.\d{3}Z$/, "Z");
  }

  function providerSmokeOrigin() {
    const origin = window.location?.origin;
    if (typeof origin === "string" && origin && origin !== "null") {
      return origin;
    }
    const href = window.location?.href;
    if (typeof href === "string") {
      const match = href.match(/^([a-zA-Z][a-zA-Z0-9+.-]*:\/\/[^/?#]+)/);
      if (match) {
        return match[1];
      }
    }
    return "framkey://local-dapp";
  }

  function providerSmokeAuthority(origin) {
    const withoutScheme = String(origin).replace(/^[a-zA-Z][a-zA-Z0-9+.-]*:\/\//, "");
    return withoutScheme.split(/[/?#]/, 1)[0] || "local-dapp";
  }

  function normalizeChainIdHex(chainIdHex) {
    if (typeof chainIdHex !== "string" || !/^0x[0-9a-fA-F]+$/.test(chainIdHex)) {
      return null;
    }
    return `0x${BigInt(chainIdHex).toString(16)}`;
  }

  const provider = new FramKeyProvider();

  window.framkey = provider;
  window.framkeyRunProviderSmoke = () =>
    runRemoteProviderSmoke(provider, {
      mode: "read",
      source: "manual_read_probe",
    });
  if (!window.ethereum) {
    window.ethereum = provider;
  }
  recordTelemetry("provider_injected", {
    provider: PROVIDER_INFO.rdns,
    ethereumAssigned: window.ethereum === provider,
  });

  function announceProvider() {
    recordTelemetry("eip6963_announce_provider", {
      provider: PROVIDER_INFO.rdns,
      ethereumAssigned: window.ethereum === provider,
    });
    window.dispatchEvent(
      new CustomEvent("eip6963:announceProvider", {
        detail: {
          info: PROVIDER_INFO,
          provider,
        },
      }),
    );
  }

  announceProvider();
  window.addEventListener("eip6963:requestProvider", () => {
    recordTelemetry("eip6963_request_provider", {
      provider: PROVIDER_INFO.rdns,
    });
    announceProvider();
  });
  setTimeout(() => {
    runRemoteProviderSmoke(provider).catch((error) => {
      recordTelemetry("provider_smoke_failed", providerSmokeError("remote_smoke", error));
    });
  }, 1000);
})();
