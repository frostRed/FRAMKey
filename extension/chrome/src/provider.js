class FramKeyProviderRpcError extends Error {
  constructor(error) {
    super(error?.message ?? "FRAMKey provider request failed");
    this.name = "FramKeyProviderRpcError";
    this.code = error?.code ?? 4200;
    this.data = error?.data;
  }
}

class FramKeyProvider {
  constructor() {
    this.isFramKey = true;
    this._nextId = 1;
    this._pending = new Map();
    this._listeners = new Map();

    window.addEventListener("message", (event) => {
      if (event.source !== window || event.data?.target !== "framkey-provider") {
        return;
      }

      const pending = this._pending.get(event.data.id);
      if (!pending) {
        return;
      }

      this._pending.delete(event.data.id);
      const response = event.data.response;
      if (response?.error) {
        pending.reject(new FramKeyProviderRpcError(response.error));
      } else {
        pending.resolve(response?.result);
      }
    });
  }

  request(args) {
    if (!args || typeof args.method !== "string") {
      return Promise.reject(
        new FramKeyProviderRpcError({
          code: -32602,
          message: "FRAMKey request requires a string method",
        }),
      );
    }

    const id = `framkey_${this._nextId++}`;

    return new Promise((resolve, reject) => {
      this._pending.set(id, { resolve, reject });
      window.postMessage(
        {
          target: "framkey-content",
          id,
          message: {
            id,
            method: args.method,
            params: args.params ?? [],
          },
        },
        window.location.origin,
      );
    });
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

  removeListener(eventName, listener) {
    const listeners = this._listeners.get(eventName);
    if (listeners) {
      listeners.delete(listener);
    }
    return this;
  }

  emit(eventName, ...args) {
    const listeners = this._listeners.get(eventName);
    if (!listeners) {
      return false;
    }
    for (const listener of listeners) {
      listener(...args);
    }
    return true;
  }
}

const provider = new FramKeyProvider();

window.framkey = provider;
if (!window.ethereum) {
  window.ethereum = provider;
}

function announceProvider() {
  window.dispatchEvent(
    new CustomEvent("eip6963:announceProvider", {
      detail: {
        info: {
          uuid: "b7b46ee4-48bc-4050-a02f-000000000001",
          name: "FRAMKey",
          icon: "",
          rdns: "dev.framkey",
        },
        provider,
      },
    }),
  );
}

announceProvider();
window.addEventListener("eip6963:requestProvider", announceProvider);
