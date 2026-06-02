(function () {
  "use strict";

  const PROVIDER_INFO = {
    uuid: "b7b46ee4-48bc-4050-a02f-000000000001",
    name: "FRAMKey",
    icon:
      "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect width='64' height='64' rx='12' fill='%23166c4d'/%3E%3Cpath d='M16 18h32v8H26v8h18v8H26v14H16V18z' fill='white'/%3E%3Cpath d='M39 34l9 11h-8l-8-11h7z' fill='%23d8f5e6'/%3E%3C/svg%3E",
    rdns: "dev.framkey",
  };
  const PROVIDER_SMOKE_TIMEOUT_MS = 30_000;
  const PROVIDER_SMOKE_MESSAGE_HEX = "0x4652414d4b65792072656d6f746520736d6f6b65";
  const PROVIDER_SMOKE_TX_TO = "0x0000000000000000000000000000000000000001";
  const PROVIDER_SMOKE_PERMIT_TOKEN = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
  const PROVIDER_SMOKE_PERMIT_SPENDER = "0x000000000022d473030f116ddee9f6b43ac78ba3";
  const PROVIDER_SMOKE_PERMIT_RECIPIENT = "0x0000000000000000000000000000000000000002";

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
          PROVIDER_SMOKE_MESSAGE_HEX,
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
    return {
      domain: {
        name: "Permit2",
        chainId,
        verifyingContract: PROVIDER_SMOKE_PERMIT_SPENDER,
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
          expiration: "1900000000",
          nonce: "0",
        },
        spender: PROVIDER_SMOKE_PERMIT_RECIPIENT,
        sigDeadline: "1900000100",
      },
    };
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
