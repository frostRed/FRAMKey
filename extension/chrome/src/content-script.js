const script = document.createElement("script");
script.src = chrome.runtime.getURL("src/provider.js");
script.type = "module";
script.onload = () => script.remove();
(document.head || document.documentElement).appendChild(script);

window.addEventListener("message", (event) => {
  if (event.source !== window || event.data?.target !== "framkey-content") {
    return;
  }

  const request = event.data.message ?? {};
  chrome.runtime.sendMessage(
    {
      id: request.id,
      method: request.method,
      params: request.params ?? [],
      origin: window.location.origin,
    },
    (response) => {
      const providerResponse = chrome.runtime.lastError
        ? {
            id: request.id,
            error: {
              code: 4900,
              message: chrome.runtime.lastError.message,
            },
          }
        : response;

      window.postMessage(
        {
          target: "framkey-provider",
          id: event.data.id,
          response: providerResponse,
        },
        window.location.origin,
      );
    },
  );
});
