const NATIVE_HOST = "dev.framkey.native_host";
const ORIGIN_PREFIX = "origin:";
const SIGNING_METHODS = new Set([
  "eth_sendTransaction",
  "eth_sign",
  "eth_signTransaction",
  "eth_signTypedData",
  "eth_signTypedData_v1",
  "eth_signTypedData_v3",
  "eth_signTypedData_v4",
  "personal_sign",
]);

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  handleProviderMessage(message, sender).then(sendResponse);
  return true;
});

async function handleProviderMessage(message, sender) {
  const id = typeof message?.id === "string" ? message.id : "unknown";
  const origin = trustedOrigin(message, sender);
  const method = message?.method;

  try {
    if (typeof method !== "string") {
      throw providerError(-32602, "FRAMKey request method must be a string");
    }
    if (!origin) {
      throw providerError(4100, "FRAMKey could not determine request origin");
    }
    if (SIGNING_METHODS.has(method)) {
      throw providerError(4200, `${method} is blocked by the read-only FRAMKey bridge`);
    }

    let result;
    switch (method) {
      case "eth_requestAccounts":
        result = await requestAccounts(id, origin);
        break;
      case "eth_accounts":
        result = await accounts(origin);
        break;
      case "eth_chainId":
        result = await nativeRequest(id, {
          method,
          params: message.params ?? [],
          origin,
        });
        break;
      case "wallet_getCapabilities":
      case "framkey_getStatus":
        result = await status(id, origin);
        break;
      default:
        throw providerError(4200, `FRAMKey does not support ${method}`);
    }

    return { id, result };
  } catch (error) {
    return {
      id,
      error: normalizeError(error),
    };
  }
}

async function requestAccounts(id, origin) {
  const account = await nativeRequest(id, {
    method: "framkey_getAccount",
    params: [],
    origin,
  });
  await chrome.storage.local.set({
    [originKey(origin)]: {
      address: account.address,
      chainId: account.chainId,
      connectedAt: new Date().toISOString(),
    },
  });
  return [account.address];
}

async function accounts(origin) {
  const stored = await chrome.storage.local.get(originKey(origin));
  const grant = stored[originKey(origin)];
  return grant?.address ? [grant.address] : [];
}

async function status(id, origin) {
  const [nativeStatus, connectedAccounts] = await Promise.all([
    nativeRequest(id, {
      method: "framkey_getStatus",
      params: [],
      origin,
    }),
    accounts(origin),
  ]);
  return {
    ...nativeStatus,
    origin,
    connectedAccounts,
  };
}

function nativeRequest(id, request) {
  return new Promise((resolve, reject) => {
    chrome.runtime.sendNativeMessage(
      NATIVE_HOST,
      {
        id,
        method: request.method,
        params: request.params ?? [],
        origin: request.origin,
      },
      (response) => {
        if (chrome.runtime.lastError) {
          reject(providerError(4900, chrome.runtime.lastError.message));
          return;
        }
        if (!response) {
          reject(providerError(4900, "FRAMKey native host returned no response"));
          return;
        }
        if (response.error) {
          reject(ipcErrorToProviderError(response.error));
          return;
        }
        resolve(response.result);
      },
    );
  });
}

function trustedOrigin(message, sender) {
  if (sender?.url) {
    try {
      return new URL(sender.url).origin;
    } catch (_error) {
      return undefined;
    }
  }
  return typeof message?.origin === "string" ? message.origin : undefined;
}

function originKey(origin) {
  return `${ORIGIN_PREFIX}${origin}`;
}

function ipcErrorToProviderError(error) {
  const code = {
    USER_REJECTED: 4001,
    DANGEROUS_SIGNATURE_BLOCKED: 4200,
    UNSUPPORTED_METHOD: 4200,
    TOUCH_ID_FAILED: 4001,
    CARD_NOT_FOUND: 4900,
    CARD_READ_FAILED: 4900,
    KEYCHAIN_ITEM_NOT_FOUND: 4900,
    VAULT_CORRUPTED: 4900,
    UNSUPPORTED_CHAIN: 4901,
  }[error.code] ?? 4900;

  return providerError(code, error.message, { ipcCode: error.code });
}

function providerError(code, message, data) {
  const error = new Error(message);
  error.code = code;
  error.data = data;
  return error;
}

function normalizeError(error) {
  return {
    code: Number.isInteger(error?.code) ? error.code : 4900,
    message: error?.message ?? "FRAMKey request failed",
    data: error?.data,
  };
}
