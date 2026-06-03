const bridgeState = document.querySelector("#bridge-state");
const runtimeSummary = document.querySelector("#runtime-summary");
const chainId = document.querySelector("#chain-id");
const networkName = document.querySelector("#network-name");
const accountAddress = document.querySelector("#account-address");
const accountBalance = document.querySelector("#account-balance");
const walletHomeBalance = document.querySelector("#wallet-home-balance");
const walletHomeAddress = document.querySelector("#wallet-home-address");
const walletHomeNetwork = document.querySelector("#wallet-home-network");
const walletHomeReadiness = document.querySelector("#wallet-home-readiness");
const walletHomeGuidance = document.querySelector("#wallet-home-guidance");
const walletHomeAccount = document.querySelector("#wallet-home-account");
const walletHomeRpc = document.querySelector("#wallet-home-rpc");
const walletHomeAssets = document.querySelector("#wallet-home-assets");
const walletHomeSecurity = document.querySelector("#wallet-home-security");
const walletMode = document.querySelector("#wallet-mode");
const rpcStatus = document.querySelector("#rpc-status");
const rpcHealthSummary = document.querySelector("#rpc-health-summary");
const rpcHealthProvider = document.querySelector("#rpc-health-provider");
const rpcHealthChain = document.querySelector("#rpc-health-chain");
const rpcHealthBlock = document.querySelector("#rpc-health-block");
const rpcHealthLatency = document.querySelector("#rpc-health-latency");
const rpcHealthUpdated = document.querySelector("#rpc-health-updated");
const rpcHealthDetail = document.querySelector("#rpc-health-detail");
const device = document.querySelector("#device");
const signerHelper = document.querySelector("#signer-helper");
const networkSelect = document.querySelector("#network-select");
const portfolioSummary = document.querySelector("#portfolio-summary");
const portfolioBlock = document.querySelector("#portfolio-block");
const portfolioNative = document.querySelector("#portfolio-native");
const portfolioTokenCount = document.querySelector("#portfolio-token-count");
const portfolioUpdated = document.querySelector("#portfolio-updated");
const portfolioAssets = document.querySelector("#portfolio-assets");
const nativeSendForm = document.querySelector("#native-send-form");
const nativeSendTo = document.querySelector("#native-send-to");
const nativeSendAmount = document.querySelector("#native-send-amount");
const nativeSendSubmitButton = document.querySelector("#native-send-submit");
const nativeSendStatus = document.querySelector("#native-send-status");
const nativeSendDetail = document.querySelector("#native-send-detail");
const tokenSendForm = document.querySelector("#token-send-form");
const tokenSendTo = document.querySelector("#token-send-to");
const tokenSendAmount = document.querySelector("#token-send-amount");
const tokenSendSubmitButton = document.querySelector("#token-send-submit");
const tokenSendStatus = document.querySelector("#token-send-status");
const tokenSendSelected = document.querySelector("#token-send-selected");
const tokenSendDetail = document.querySelector("#token-send-detail");
const activityCount = document.querySelector("#activity-count");
const activityReceiptTracking = document.querySelector("#activity-receipt-tracking");
const activityPersistence = document.querySelector("#activity-persistence");
const transactionActivity = document.querySelector("#transaction-activity");
const activityHomeTitle = document.querySelector("#activity-home-title");
const activityHomeSubtitle = document.querySelector("#activity-home-subtitle");
const activityHomeState = document.querySelector("#activity-home-state");
const activityLatestOutcome = document.querySelector("#activity-latest-outcome");
const activityLatestOutcomeDetail = document.querySelector("#activity-latest-outcome-detail");
const activityReceiptState = document.querySelector("#activity-receipt-state");
const activityReceiptDetail = document.querySelector("#activity-receipt-detail");
const activityStorageState = document.querySelector("#activity-storage-state");
const activityStorageDetail = document.querySelector("#activity-storage-detail");
const activityNextAction = document.querySelector("#activity-next-action");
const activityNextDetail = document.querySelector("#activity-next-detail");
const capabilities = document.querySelector("#capabilities");
const output = document.querySelector("#output");
const reviewCount = document.querySelector("#review-count");
const reviewList = document.querySelector("#review-list");
const connectedSites = document.querySelector("#connected-sites");
const providerEventCount = document.querySelector("#provider-event-count");
const providerEvents = document.querySelector("#provider-events");
const dappUrl = document.querySelector("#dapp-url");
const dappCurrentTarget = document.querySelector("#dapp-current-target");
const dappCurrentOrigin = document.querySelector("#dapp-current-origin");
const dappCurrentUrl = document.querySelector("#dapp-current-url");
const dappLoadStatus = document.querySelector("#dapp-load-status");
const dappUpdated = document.querySelector("#dapp-updated");
const defiHomeTitle = document.querySelector("#defi-home-title");
const defiHomeSubtitle = document.querySelector("#defi-home-subtitle");
const defiHomeState = document.querySelector("#defi-home-state");
const defiReviewCallout = document.querySelector("#defi-review-callout");
const defiReviewCalloutTitle = document.querySelector("#defi-review-callout-title");
const defiReviewCalloutDetail = document.querySelector("#defi-review-callout-detail");
const defiCurrentApp = document.querySelector("#defi-current-app");
const defiCurrentOrigin = document.querySelector("#defi-current-origin");
const defiAccessState = document.querySelector("#defi-access-state");
const defiAccessDetail = document.querySelector("#defi-access-detail");
const defiNextAction = document.querySelector("#defi-next-action");
const defiNextDetail = document.querySelector("#defi-next-detail");
const defiLatestResult = document.querySelector("#defi-latest-result");
const defiLatestResultDetail = document.querySelector("#defi-latest-result-detail");
const defiCockpitApp = document.querySelector("#defi-cockpit-app");
const defiCockpitAccess = document.querySelector("#defi-cockpit-access");
const defiCockpitNext = document.querySelector("#defi-cockpit-next");
const defiCockpitResult = document.querySelector("#defi-cockpit-result");
const defiPrimaryApproval = document.querySelector("#defi-primary-approval");
const defiPrimaryApprovalTitle = document.querySelector("#defi-primary-approval-title");
const defiPrimaryApprovalDetail = document.querySelector("#defi-primary-approval-detail");
const defiPrimaryApprovalImpact = document.querySelector("#defi-primary-approval-impact");
const defiPrimaryApprovalApprove = document.querySelector("#defi-primary-approval-approve");
const defiPrimaryApprovalReject = document.querySelector("#defi-primary-approval-reject");
const defiPrimaryApprovalDetails = document.querySelector("#defi-primary-approval-details");
const defiStepRpc = document.querySelector("#defi-step-rpc");
const defiStepProvider = document.querySelector("#defi-step-provider");
const defiStepConnect = document.querySelector("#defi-step-connect");
const defiStepReview = document.querySelector("#defi-step-review");
const vaultGeneration = document.querySelector("#vault-generation");
const recoveryOutDir = document.querySelector("#recovery-out-dir");
const confirmOverwrite = document.querySelector("#confirm-overwrite");
const createConfirmRow = document.querySelector(".create-flow .restore-confirm-row");
const createVaultWriteNote = document.querySelector("#create-vault-write-note");
const createVaultStatus = document.querySelector("#create-vault-status");
const vaultBackupPath = document.querySelector("#vault-backup-path");
const recoveryFilePaths = document.querySelector("#recovery-file-paths");
const recoverySetSummary = document.querySelector("#recovery-set-summary");
const recoverySetPolicy = document.querySelector("#recovery-set-policy");
const recoverOverwrite = document.querySelector("#recover-overwrite");
const recoveryPlan = document.querySelector("#recovery-plan");
const recoveryHomeTitle = document.querySelector("#recovery-home-title");
const recoveryHomeSubtitle = document.querySelector("#recovery-home-subtitle");
const recoveryHomeState = document.querySelector("#recovery-home-state");
const recoveryHomePack = document.querySelector("#recovery-home-pack");
const recoveryHomePlacement = document.querySelector("#recovery-home-placement");
const recoveryHomeDrill = document.querySelector("#recovery-home-drill");
const recoveryPanel = document.querySelector(".recovery-panel");
const restoreStepFiles = document.querySelector("#restore-step-files");
const restoreStepWrite = document.querySelector("#restore-step-write");
const restoreWriteSummary = document.querySelector("#restore-write-summary");
const restoreSelectedFiles = document.querySelector("#restore-selected-files");
const restoreWriteDetail = document.querySelector("#restore-write-detail");
const restoreSchemeCards = Array.from(document.querySelectorAll("[data-recovery-scheme]"));
const restoreFileSlots = document.querySelector("#restore-file-slots");
const sessionSummary = document.querySelector("#session-summary");
const readinessGrid = document.querySelector("#readiness-grid");
const sessionDapp = document.querySelector("#session-dapp");
const sessionAccountGrant = document.querySelector("#session-account-grant");
const sessionProvider = document.querySelector("#session-provider");
const sessionSign = document.querySelector("#session-sign");
const sessionTransaction = document.querySelector("#session-transaction");
const sessionNextAction = document.querySelector("#session-next-action");
const compatibilitySummary = document.querySelector("#compatibility-summary");
const compatibilityGrid = document.querySelector("#compatibility-grid");
const workspaceTabs = Array.from(document.querySelectorAll("[data-workspace-tab]"));
const workspacePanels = Array.from(document.querySelectorAll("[data-workspace]"));
const workspaceCounts = new Map(
  Array.from(document.querySelectorAll("[data-workspace-count]")).map((item) => [
    item.dataset.workspaceCount,
    item,
  ]),
);

let trustedAutosmokeStarted = false;
let recoveryAutosmokeStarted = false;
let recoveryStateRestoreSmokeReported = false;
let activeDappTarget = "No app open";
let latestDappSession = null;
let latestStatus = null;
let latestAccount = null;
let latestRpcHealth = null;
let latestPortfolio = null;
let selectedTokenForSend = null;
let latestConnectedOrigins = [];
let latestProviderEvents = [];
let latestReviewRequests = [];
let lastPendingReviewKey = "";
let latestTransactionActivity = [];
let latestActivityPersistence = null;
let defiPrimaryApprovalRequest = null;
let transactionActivitySmokeReported = false;
let walletSendAutosmokeStarted = false;
let receiptAutoRefreshTimer = null;
let receiptAutoRefreshInFlight = false;
let lastReceiptRefreshAtUnixMs = 0;
let latestReceiptRefreshError = null;
let activeCompatibilityChecks = new Set();
let creatingVault = false;
let createVaultCompleted = false;
let recoveringVault = false;
let walletConnectionPending = null;
let walletConnectionOperationId = 0;
let walletConnectionError = null;
let keychainHelperAccessPending = false;

const COMPATIBILITY_TARGETS = [
  {
    key: "local",
    label: "Local Test",
    origin: "tauri://localhost",
    openUrl: "local",
  },
  {
    key: "uniswap",
    label: "Uniswap",
    origin: "https://app.uniswap.org",
    openUrl: "uniswap",
  },
  {
    key: "aave",
    label: "Aave",
    origin: "https://app.aave.com",
    openUrl: "aave",
  },
];
const COMPATIBILITY_CHECK_SETTLE_MS = 2_500;
const COMPATIBILITY_CHECK_REFRESH_MS = 3_500;
const TRANSACTION_RECEIPT_AUTO_REFRESH_MS = 15_000;
const TRANSACTION_RECEIPT_AUTO_REFRESH_MIN_DELAY_MS = 1_000;
const WALLET_SEND_AUTOSMOKE_RECIPIENT = "0x0000000000000000000000000000000000000001";
const WALLET_SEND_AUTOSMOKE_NATIVE_AMOUNT = "0.000000000000000001";
const WALLET_SEND_AUTOSMOKE_TOKEN_AMOUNT = "0.000001";
const TRANSACTION_ACTIVITY_FINAL_STATUSES = new Set([
  "confirmed",
  "included",
  "reverted",
  "failed",
  "rejected",
  "expired",
]);
const BACKUP_PLACEMENT_STORAGE_KEY = "framkey.backupPlacement.v1";
const WORKSPACE_STORAGE_KEY = "framkey.workspace.v1";
const RECOVERY_SCHEMES = {
  cloudPhysical: {
    key: "cloudPhysical",
    label: "Cloud + physical",
    summary: "Select iCloud, Google, and one physical backup file.",
    slots: [
      {
        key: "icloud",
        label: "iCloud",
        role: "icloud",
        bucket: "cloud",
        detail: "Choose backup-01.dat",
        empty: "No iCloud file selected",
      },
      {
        key: "google",
        label: "Google Drive",
        role: "google",
        bucket: "cloud",
        detail: "Choose backup-02.dat",
        empty: "No Google file selected",
      },
      {
        key: "physical",
        label: "Physical",
        role: "physical_unknown",
        bucket: "local",
        detail: "Choose backup-03.dat or backup-04.dat",
        empty: "No physical file selected",
      },
    ],
  },
  physicalPair: {
    key: "physicalPair",
    label: "Two physical files",
    summary: "Select the local and off-site physical backup files.",
    slots: [
      {
        key: "local",
        label: "Local",
        role: "local",
        bucket: "local",
        detail: "Choose backup-03.dat",
        empty: "No local file selected",
      },
      {
        key: "remote",
        label: "Off-site",
        role: "remote",
        bucket: "local",
        detail: "Choose backup-04.dat",
        empty: "No off-site file selected",
      },
    ],
  },
};

let latestRecoveryBackupOutcome = null;
let latestRecoveryDrillOutcome = null;
let latestRecoveryRecoverOutcome = null;
let latestRecoveryPersistence = null;
let selectedCloudRecoveryFiles = [];
let selectedLocalRecoveryFiles = [];
let activeRecoverySchemeKey = "cloudPhysical";
let selectedRecoverySlotFiles = {};
let backupPlacementState = loadBackupPlacementState();
let activeWorkspace = loadWorkspaceSelection();

const refreshStatusButton = document.querySelector("#refresh-status");
const refreshRpcHealthButton = document.querySelector("#refresh-rpc-health");
const switchNetworkButton = document.querySelector("#switch-network");
const walletActionConnectButton = document.querySelector("#wallet-action-connect");
const walletActionSendButton = document.querySelector("#wallet-action-send");
const walletActionDefiButton = document.querySelector("#wallet-action-defi");
const refreshPortfolioButton = document.querySelector("#refresh-portfolio");
const refreshActivityButton = document.querySelector("#refresh-activity");
const refreshReceiptsButton = document.querySelector("#refresh-receipts");
const connectCardButton = document.querySelector("#connect-card");
const authorizeKeychainHelperButton = document.querySelector("#authorize-keychain-helper");
const openDappButton = document.querySelector("#open-dapp");
const openCustomDappButton = document.querySelector("#open-custom-dapp");
const openUniswapButton = document.querySelector("#open-uniswap");
const openAaveButton = document.querySelector("#open-aave");
const openLocalDappButton = document.querySelector("#open-local-dapp");
const defiActionUniswapButton = document.querySelector("#defi-action-uniswap");
const defiActionAaveButton = document.querySelector("#defi-action-aave");
const defiActionConnectButton = document.querySelector("#defi-action-connect");
const dappNavBackButton = document.querySelector("#dapp-nav-back");
const dappNavForwardButton = document.querySelector("#dapp-nav-forward");
const dappNavReloadButton = document.querySelector("#dapp-nav-reload");
const dappNavHomeButton = document.querySelector("#dapp-nav-home");
const refreshConnectionsButton = document.querySelector("#refresh-connections");
const refreshProviderEventsButton = document.querySelector("#refresh-provider-events");
const clearProviderEventsButton = document.querySelector("#clear-provider-events");
const createVaultButton = document.querySelector("#create-vault");
const chooseRecoveryOutDirButton = document.querySelector("#choose-recovery-out-dir");
const clearRecoveryFilesButton = document.querySelector("#clear-recovery-files");
const recoverVaultButton = document.querySelector("#recover-vault");
const clearRecoveryPlanButton = document.querySelector("#clear-recovery-plan");
const refreshReviewButton = document.querySelector("#refresh-review");
const clearReviewButton = document.querySelector("#clear-review");

function tauriInvoke() {
  const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
  if (!invoke) {
    throw new Error("Tauri bridge is unavailable");
  }
  return invoke;
}

async function invokeCommand(command, args = {}) {
  setBusy(command);
  try {
    const response = await tauriInvoke()(command, args);
    renderEnvelope(response);
    return response;
  } catch (error) {
    renderError(error);
    throw error;
  }
}

async function invokeQuiet(command, args = {}) {
  return tauriInvoke()(command, args);
}

async function providerRequest(method, params = []) {
  return invokeCommand("framkey_provider_request", {
    request: {
      id: `trusted_ui_${Date.now()}`,
      method,
      params,
      origin: "framkey://trusted-ui",
    },
  });
}

async function providerRequestQuiet(method, params = []) {
  return invokeQuiet("framkey_provider_request", {
    request: {
      id: `trusted_ui_${Date.now()}`,
      method,
      params,
      origin: "framkey://trusted-ui",
    },
  });
}

function loadWorkspaceSelection() {
  try {
    const value = window.localStorage?.getItem(WORKSPACE_STORAGE_KEY);
    if (workspaceTabs.some((tab) => tab.dataset.workspaceTab === value)) {
      return value;
    }
  } catch {
    // Local storage is optional in bundled WebViews.
  }
  return "wallet";
}

function setActiveWorkspace(workspace, { persist = true } = {}) {
  if (!workspaceTabs.some((tab) => tab.dataset.workspaceTab === workspace)) {
    workspace = "wallet";
  }
  activeWorkspace = workspace;
  for (const tab of workspaceTabs) {
    const selected = tab.dataset.workspaceTab === workspace;
    tab.setAttribute("aria-selected", selected ? "true" : "false");
    tab.dataset.selected = selected ? "true" : "false";
  }
  for (const panel of workspacePanels) {
    panel.hidden = !panelWorkspaces(panel).includes(workspace);
  }
  if (persist) {
    try {
      window.localStorage?.setItem(WORKSPACE_STORAGE_KEY, workspace);
    } catch {
      // Selection persistence is a convenience, not runtime state.
    }
  }
  window.scrollTo(0, 0);
}

function panelWorkspaces(panel) {
  return String(panel.dataset.workspace ?? "")
    .split(/\s+/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function updateWorkspaceReviewCounts() {
  for (const count of workspaceCounts.values()) {
    count.textContent = "0";
    count.hidden = true;
  }
}

function renderProductOverview() {
  renderWalletProductOverview();
  renderDefiProductOverview();
  renderActivityProductOverview();
  renderRecoveryProductOverview();
}

function renderWalletProductOverview() {
  if (!walletHomeBalance) {
    return;
  }
  const address = latestPortfolio?.address ?? latestAccount?.address ?? null;
  const nativeBalance = latestPortfolio?.native?.balance;
  const tokens = latestPortfolio?.tokens ?? [];
  const pending = latestReviewRequests.filter((request) => request.status === "pending");
  const rpcHealthy = latestRpcHealth ? Boolean(latestRpcHealth.healthy) : Boolean(latestStatus?.rpc);
  const helperReady = Boolean(latestStatus?.wallet?.mock || latestStatus?.signerHelper?.ready);
  const connecting = walletConnectionPending === "connecting";
  const disconnecting = walletConnectionPending === "disconnecting";
  const authorizingKeychain = keychainHelperAccessPending;
  const connectionPending = connecting || disconnecting;
  const connectionError = address ? null : walletConnectionError;

  walletHomeBalance.textContent = address
    ? nativeBalance
      ? formatNativeBalance(nativeBalance)
      : accountBalance.textContent || "0 ETH"
    : "0 ETH";
  walletHomeAddress.textContent = connecting
    ? "Reading vault from card; macOS unlock follows"
    : connectionError
      ? walletConnectionErrorText(connectionError)
    : address
      ? shortAddress(address)
      : "Unlock the vault to load the account";
  walletHomeNetwork.textContent = walletHomeNetworkLabel();
  walletHomeAccount.textContent = address ? "Connected" : connectionError ? "Connection failed" : "Disconnected";
  walletHomeRpc.textContent = latestRpcHealth
    ? rpcHealthSummaryText(latestRpcHealth)
    : latestStatus?.rpc
      ? "Configured"
      : "Missing";
  walletHomeAssets.textContent =
    address == null
      ? "Disconnected"
      : latestPortfolio == null
        ? "No snapshot"
        : `${tokens.length} token${tokens.length === 1 ? "" : "s"}`;
  walletHomeSecurity.textContent = latestStatus?.wallet?.mock
    ? "Test mode"
    : helperReady
      ? "Signing ready"
      : "Signing unavailable";

  let readiness = connecting
    ? "Connecting"
    : disconnecting
      ? "Disconnecting"
      : authorizingKeychain
        ? "Preparing signing"
      : address
        ? "Connected"
        : "Disconnected";
  let tone = "good";
  if (connectionPending || authorizingKeychain) {
    tone = "warn";
  } else if (connectionError) {
    readiness = walletConnectionErrorReadiness(connectionError);
    tone = "bad";
  } else if (pending.length > 0) {
    readiness = `${pending.length} approval`;
    tone = "warn";
  } else if (!helperReady) {
    readiness = "Setup needed";
    tone = "bad";
  } else if (!rpcHealthy) {
    readiness = "RPC attention";
    tone = "warn";
  } else if (!address) {
    tone = "idle";
  }
  walletHomeReadiness.textContent = readiness;
  walletHomeReadiness.dataset.tone = tone;
  if (walletHomeGuidance) {
    walletHomeGuidance.textContent = walletHomeGuidanceText({
      address,
      authorizingKeychain,
      connecting,
      connectionError,
      disconnecting,
      helperReady,
      pendingCount: pending.length,
      rpcHealthy,
    });
  }
  walletActionConnectButton.textContent = connecting
    ? "Unlocking..."
    : disconnecting
      ? "Disconnecting..."
      : address
        ? "Disconnect"
        : "Unlock";
  walletActionConnectButton.dataset.mode = address ? "disconnect" : "connect";
  walletActionConnectButton.disabled = connectionPending;
  connectCardButton.textContent = address ? "Wallet Unlocked" : "Unlock Wallet";
  connectCardButton.disabled = connectionPending || Boolean(address);
  authorizeKeychainHelperButton.textContent = authorizingKeychain
    ? "Repairing..."
    : "Repair Signing Access";
  authorizeKeychainHelperButton.disabled =
    authorizingKeychain || connectionPending || Boolean(latestStatus?.wallet?.mock) || !helperReady;
  walletActionSendButton.disabled = !address || connectionPending;
}

function walletHomeNetworkLabel() {
  const label = latestStatus?.network?.name ?? networkName.textContent;
  return label && label !== "-" ? label : "Ethereum";
}

function walletHomeGuidanceText({
  address,
  authorizingKeychain,
  connecting,
  connectionError,
  disconnecting,
  helperReady,
  pendingCount,
  rpcHealthy,
}) {
  if (connecting) {
    return "Reading the vault and preparing the local unlock.";
  }
  if (disconnecting) {
    return "Clearing the local wallet session.";
  }
  if (authorizingKeychain) {
    return "macOS is preparing signing access for this build.";
  }
  if (connectionError) {
    return "Reconnect the vault or repair signing access from System.";
  }
  if (pendingCount > 0) {
    return "A wallet approval is waiting in this trusted window.";
  }
  if (!helperReady) {
    return "Signing setup needs attention before real approvals.";
  }
  if (!rpcHealthy) {
    return "Network status needs a check before DeFi activity.";
  }
  if (address) {
    return "Ready for trusted sends and DeFi approvals.";
  }
  return "Unlock the vault to load the wallet.";
}

function renderDefiProductOverview() {
  if (!defiHomeTitle) {
    return;
  }
  const connectedOrigin = latestConnectedOrigins[0] ?? null;
  const providerInjected = latestProviderEvents.some((event) => event.kind === "provider_injected");
  const pending = latestReviewRequests.filter((request) => request.status === "pending");
  const rpcReady = latestRpcHealth ? Boolean(latestRpcHealth.healthy) : Boolean(latestStatus?.rpc);
  const dappOpen = Boolean(latestDappSession?.open);
  const target = dappOpen ? (latestDappSession?.targetLabel ?? activeDappTarget ?? "App") : "No app open";
  const origin = connectedOrigin ?? latestDappSession?.origin ?? null;
  const latestTx = lastItem(latestReviewRequests.filter((request) => request.kind === "transaction"));
  const nextAction = nextSessionAction({
    walletReady: Boolean(latestStatus?.wallet),
    rpcReady,
    providerReady: providerInjected,
    accountReady: Boolean(latestAccount?.address || connectedOrigin),
    connectedOrigin,
    pending,
    latestTransaction: latestTx,
    latestActivity: latestTransactionActivity[0],
  });

  defiHomeTitle.textContent = !dappOpen || target === "Local Test" ? "Choose a DeFi app" : target;
  defiHomeSubtitle.textContent = connectedOrigin
    ? `${shortOrigin(connectedOrigin)} has wallet access. Every signature and transaction still stops here.`
    : dappOpen
      ? "Connect from the app, then approve wallet access in FRAMKey."
      : "Pick an app or open a trusted URL.";

  let state = "Ready";
  let tone = "good";
  if (pending.length > 0) {
    state = `${pending.length} approval`;
    tone = "warn";
  } else if (!rpcReady) {
    state = "Network issue";
    tone = "bad";
  } else if (!providerInjected) {
    state = "Open app";
    tone = "busy";
  } else if (!connectedOrigin && origin) {
    state = "Connect";
    tone = "warn";
  }
  defiHomeState.textContent = state;
  defiHomeState.dataset.tone = tone;
  renderDefiReviewCallout(pending);
  renderDefiCockpit({
    connectedOrigin,
    dappOpen,
    origin,
    pending,
    providerInjected,
    rpcReady,
    target,
    nextAction,
    latestActivity: latestTransactionActivity[0],
  });
  renderDefiPrimaryApproval(pending);

  setJourneyStep(defiStepRpc, rpcReady ? "good" : "bad", rpcReady ? "Ready" : "Check network");
  setJourneyStep(
    defiStepProvider,
    providerInjected ? "good" : "warn",
    providerInjected ? "Ready" : "Open app",
  );
  setJourneyStep(
    defiStepConnect,
    connectedOrigin ? "good" : "warn",
    connectedOrigin ? "Connected" : "Not connected",
  );
  setJourneyStep(
    defiStepReview,
    pending.length > 0 ? "warn" : "good",
    pending.length > 0 ? `${pending.length} pending` : "Clear",
  );
}

function renderDefiReviewCallout(pending) {
  if (!defiReviewCallout) {
    return;
  }
  const request = pending[0] ?? null;
  defiReviewCallout.hidden = !request || Boolean(defiPrimaryApproval);
  if (!request) {
    return;
  }
  const count = pending.length;
  defiReviewCalloutTitle.textContent =
    count === 1 ? reviewIntentTitle(request) : `${count} approval requests`;
  defiReviewCalloutDetail.textContent = `${request.origin ?? "unknown origin"} · ${
    request.method ?? "wallet request"
  }`;
}

function renderDefiCockpit({
  connectedOrigin,
  dappOpen,
  origin,
  pending,
  providerInjected,
  rpcReady,
  target,
  nextAction,
  latestActivity,
}) {
  if (!defiCurrentApp) {
    return;
  }

  setCardTone(defiCockpitApp, dappOpen ? (providerInjected ? "good" : "warn") : "idle");
  defiCurrentApp.textContent = dappOpen ? target : "No app open";
  defiCurrentOrigin.textContent = dappOpen
    ? origin
      ? `${shortOrigin(origin)}`
      : "App loading"
    : "Choose Uniswap, Aave, or enter a URL.";

  const connected = Boolean(connectedOrigin);
  setCardTone(defiCockpitAccess, connected ? "good" : dappOpen ? "warn" : "idle");
  defiAccessState.textContent = connected ? "Connected" : "Not connected";
  defiAccessDetail.textContent = connected
    ? `${shortOrigin(connectedOrigin)} can ask for approvals. Nothing signs automatically.`
    : dappOpen
      ? "Waiting for an account request from the app."
      : "No app can see the wallet yet.";

  const nextTone = pending.length > 0 ? "warn" : !rpcReady ? "bad" : providerInjected && connected ? "good" : "warn";
  setCardTone(defiCockpitNext, nextTone);
  defiNextAction.textContent = pending.length > 0 ? "Review approval" : nextAction;
  defiNextDetail.textContent =
    pending.length > 0
      ? `${pending.length} approval${pending.length === 1 ? "" : "s"} waiting.`
      : "Access, signatures, and transactions are approved separately.";

  setCardTone(defiCockpitResult, activityOutcomeTone(latestActivity));
  defiLatestResult.textContent = latestActivity
    ? transactionActivityStatusLabel(latestActivity)
    : pending.length > 0
      ? "Approval pending"
      : "No app activity";
  defiLatestResultDetail.textContent = latestActivity
    ? transactionActivityDetail(latestActivity)
    : pending.length > 0
      ? "Approve or reject to continue."
      : "Approvals and transaction results appear here.";
}

function renderDefiPrimaryApproval(pending) {
  if (!defiPrimaryApproval) {
    return;
  }
  const request = pending[0] ?? null;
  defiPrimaryApprovalRequest = request;
  defiPrimaryApproval.hidden = !request;
  if (!request) {
    return;
  }

  defiPrimaryApproval.dataset.tone = primaryApprovalTone(request);
  defiPrimaryApprovalTitle.textContent = reviewIntentTitle(request);
  defiPrimaryApprovalDetail.textContent = primaryApprovalDetail(request);
  defiPrimaryApprovalImpact.replaceChildren(
    ...primaryApprovalBadges(request).map((badge) => approvalBadge(badge.label, badge.tone)),
  );

  const approveAction = approveActionForRequest(request);
  defiPrimaryApprovalApprove.textContent = approveAction.label;
  defiPrimaryApprovalApprove.disabled = approveAction.disabled;
  defiPrimaryApprovalApprove.title = approveAction.disabledReason ?? "";
  if (approveAction.tone) {
    defiPrimaryApprovalApprove.dataset.tone = approveAction.tone;
  } else {
    delete defiPrimaryApprovalApprove.dataset.tone;
  }
  defiPrimaryApprovalReject.disabled = false;
}

function primaryApprovalTone(request) {
  if (request.kind === "transaction") {
    return transactionRiskTone(request.summary?.risk, request.summary?.policy);
  }
  if (request.kind === "typed_data" && !typedDataSigningAllowed(request)) {
    return "bad";
  }
  if (request.kind === "personal_sign" || request.kind === "typed_data") {
    return "warn";
  }
  return "good";
}

function primaryApprovalDetail(request) {
  const origin = shortOrigin(request.origin) || "This app";
  const summary = request.summary ?? {};
  if (request.kind === "account_connection") {
    return `${origin} wants to see your wallet address. This does not allow signing or moving funds.`;
  }
  if (request.kind === "network_switch") {
    const action = summary.intent === "add_network" ? "add a supported network" : "switch the active network";
    return `${origin} wants to ${action}. FRAMKey ignores dApp-provided RPC URLs and uses the trusted endpoint.`;
  }
  if (request.kind === "watch_asset") {
    return `${origin} wants to add ${valueOrDash(summary.symbol)} to the trusted Assets view. This does not grant token control.`;
  }
  if (request.kind === "personal_sign") {
    const message = summary.message ?? {};
    return `${origin} wants a wallet signature for: ${valueOrDash(message.preview ?? message.utf8Preview)}`;
  }
  if (request.kind === "typed_data") {
    const typedData = summary.typedData ?? {};
    const permit = typedData.permit ?? {};
    return `${origin} wants token permission: ${typedPermitIntentLabel(
      typedData.intent ?? typedData.primaryType,
    )}: ${typedPermitAmountLabel(permit)} for ${shortAddress(permit.spender)}.`;
  }
  if (request.kind === "transaction") {
    const guidance = summary.guidance;
    if (guidance?.message) {
      return guidance.message;
    }
    const impact = summary.impact?.title;
    const risk = summary.risk?.title;
    return [risk, impact].filter(Boolean).join(". ") || `${origin} wants to submit a transaction.`;
  }
  return `${origin} is requesting a wallet action.`;
}

function primaryApprovalBadges(request) {
  const summary = request.summary ?? {};
  const badges = [{ label: shortOrigin(request.origin) || "Unknown app", tone: "idle" }];
  if (request.kind === "transaction") {
    badges.push({
      label: transactionRiskLevel(summary.risk, summary.policy),
      tone: transactionRiskTone(summary.risk, summary.policy),
    });
    badges.push({
      label: transactionRiskActionLabel(summary.risk?.action, summary.policy),
      tone: transactionRiskTone(summary.risk, summary.policy),
    });
    if (summary.impact) {
      badges.push({
        label: transactionImpactBadgeLabel(summary.impact),
        tone: summary.impact.approvalCount > 0 ? "warn" : "good",
      });
    }
    const protocol = transactionProtocolLabel(summary.simulation);
    if (protocol !== "-") {
      badges.push({ label: protocol, tone: "idle" });
    }
    if (summary.trust?.title) {
      badges.push({ label: summary.trust.title, tone: transactionTrustTone(summary.trust) });
    }
    return badges.slice(0, 5);
  }
  if (request.kind === "typed_data") {
    const typedData = summary.typedData ?? {};
    const permit = typedData.permit ?? {};
    badges.push({ label: typedPermitIntentLabel(typedData.intent ?? typedData.primaryType), tone: "warn" });
    badges.push({ label: typedPermitTokenLabel(permit), tone: "idle" });
    badges.push({
      label: typedDataSigningAllowed(request) ? "Approval required" : "Blocked",
      tone: typedDataSigningAllowed(request) ? "warn" : "bad",
    });
    return badges;
  }
  if (request.kind === "account_connection") {
    badges.push({ label: "Address only", tone: "good" });
    badges.push({ label: "No signing grant", tone: "good" });
    return badges;
  }
  if (request.kind === "personal_sign") {
    badges.push({ label: "Signature", tone: "warn" });
    badges.push({ label: `${summary.message?.bytes ?? summary.message?.chars ?? 0} bytes`, tone: "idle" });
    return badges;
  }
  badges.push({ label: request.method ?? "wallet request", tone: "idle" });
  return badges;
}

function transactionImpactBadgeLabel(impact) {
  const transferCount = impact?.transferCount ?? 0;
  const approvalCount = impact?.approvalCount ?? 0;
  if (approvalCount > 0 && transferCount > 0) {
    return `${transferCount} transfer, ${approvalCount} approval`;
  }
  if (approvalCount > 0) {
    return `${approvalCount} token approval${approvalCount === 1 ? "" : "s"}`;
  }
  if (transferCount > 0) {
    return `${transferCount} transfer${transferCount === 1 ? "" : "s"}`;
  }
  return "No asset movement";
}

function approvalBadge(label, tone = "idle") {
  const badge = document.createElement("span");
  badge.dataset.tone = tone;
  badge.textContent = label;
  return badge;
}

function setCardTone(element, tone) {
  if (element) {
    element.dataset.tone = tone;
  }
}

function setJourneyStep(element, tone, label) {
  if (!element) {
    return;
  }
  element.dataset.tone = tone;
  const detail = element.querySelector("small");
  if (detail) {
    detail.textContent = label;
  }
}

function renderRecoveryProductOverview() {
  if (!recoveryHomeTitle) {
    return;
  }
  const shares = currentRecoveryShares();
  const placement = recoveryReadinessState(shares);
  const backupCreated = Boolean(latestRecoveryBackupOutcome);

  recoveryHomeTitle.textContent = backupCreated ? "Recovery is in progress" : "Protect this wallet";
  recoveryHomeSubtitle.textContent = backupCreated
    ? placement.nextAction
    : "Create the backup files and put them in the right places.";
  recoveryHomeState.textContent = placement.badge;
  recoveryHomeState.dataset.tone = placement.tone;
  recoveryHomePack.textContent = backupCreated ? "Created" : "Waiting";
  recoveryHomePlacement.textContent = placement.badge;
  recoveryHomeDrill.textContent = placement.tone === "good" ? "Ready" : "Waiting";
}

function renderActivityProductOverview() {
  if (!activityHomeTitle) {
    return;
  }

  const latest = latestTransactionActivity[0] ?? null;
  const outcome = activityOutcomeSummary(latest);
  const receipt = activityReceiptSummary(latestTransactionActivity);
  const storage = activityStorageSummary(latestActivityPersistence);
  const next = activityNextActionSummary(latest, receipt);

  activityHomeTitle.textContent = latest ? outcome.title : "No wallet activity yet";
  activityHomeSubtitle.textContent = latest
    ? outcome.detail
    : "Approved dApp transactions and trusted sends will appear here.";
  activityHomeState.textContent = latest ? outcome.state : "Empty";
  activityHomeState.dataset.tone = latest ? outcome.tone : "idle";

  activityLatestOutcome.textContent = outcome.title;
  activityLatestOutcomeDetail.textContent = outcome.detail;
  setCardTone(document.querySelector("#activity-cockpit-outcome"), outcome.tone);

  activityReceiptState.textContent = receipt.title;
  activityReceiptDetail.textContent = receipt.detail;
  setCardTone(document.querySelector("#activity-cockpit-receipt"), receipt.tone);

  activityStorageState.textContent = storage.title;
  activityStorageDetail.textContent = storage.detail;
  setCardTone(document.querySelector("#activity-cockpit-storage"), storage.tone);

  activityNextAction.textContent = next.title;
  activityNextDetail.textContent = next.detail;
  setCardTone(document.querySelector("#activity-cockpit-next"), next.tone);
}

function activityOutcomeSummary(item) {
  if (!item) {
    return {
      title: "No transaction",
      detail: "Nothing has been approved or submitted.",
      state: "Empty",
      tone: "idle",
    };
  }
  const status = transactionActivityStatusLabel({
    status: item.receipt?.status ?? item.receiptStatus ?? item.status,
  });
  const hash = item.transactionHash ? ` · ${shortHash(item.transactionHash)}` : "";
  return {
    title: status,
    detail: transactionActivityDetail(item) + hash,
    state: status,
    tone: activityOutcomeTone(item),
  };
}

function transactionActivityDetail(item) {
  if (!item) {
    return "No app activity";
  }
  if (item.error) {
    return `${shortOrigin(item.origin) || "unknown app"} · ${item.error}`;
  }
  const call = item.call ?? "eth_sendTransaction";
  const value = item.value ? ` · ${formatNativeBalance(item.value)}` : "";
  const receipt = item.transactionHash ? ` · ${transactionReceiptLabel(item)}` : "";
  return `${shortOrigin(item.origin) || "unknown app"} · ${call}${value}${receipt}`;
}

function activityOutcomeTone(item) {
  if (!item) {
    return "idle";
  }
  const status = item.receipt?.status ?? item.receiptStatus ?? item.status;
  if (["confirmed", "included", "broadcast"].includes(status)) {
    return "good";
  }
  if (["review_pending", "approved"].includes(status)) {
    return "warn";
  }
  if (["failed", "rejected", "expired", "reverted"].includes(status)) {
    return "bad";
  }
  return "idle";
}

function activityReceiptSummary(items) {
  const refreshable = refreshableReceiptItems(items);
  const latestReceipt = latestTransactionReceiptState(items);
  if (receiptAutoRefreshInFlight) {
    return {
      title: "Checking receipt",
      detail: "FRAMKey is asking the trusted RPC for the latest transaction status.",
      tone: "busy",
    };
  }
  if (latestReceiptRefreshError && refreshable.length > 0) {
    return {
      title: "Receipt check needs retry",
      detail: latestReceiptRefreshError.message ?? "Receipt status is unavailable right now.",
      tone: "warn",
    };
  }
  if (refreshable.length > 0) {
    return {
      title: `${refreshable.length} pending`,
      detail: latestReceipt?.checkedAt
        ? `Waiting for confirmation; last checked ${formatTime(latestReceipt.checkedAt)}.`
        : "Waiting for the network to include the transaction.",
      tone: "warn",
    };
  }
  if (latestReceipt?.status === "confirmed" || latestReceipt?.status === "included") {
    return {
      title: "Confirmed",
      detail: latestReceipt.checkedAt
        ? `Latest receipt checked ${formatTime(latestReceipt.checkedAt)}.`
        : "Latest receipt is confirmed.",
      tone: "good",
    };
  }
  if (latestReceipt?.status === "reverted") {
    return {
      title: "Reverted",
      detail: latestReceipt.checkedAt
        ? `Latest receipt checked ${formatTime(latestReceipt.checkedAt)}.`
        : "The network reported a reverted transaction.",
      tone: "bad",
    };
  }
  return {
    title: "No pending receipt",
    detail: "Broadcast transactions are checked automatically.",
    tone: "idle",
  };
}

function activityStorageSummary(persistence) {
  if (persistence?.warning) {
    return {
      title: "Save needs attention",
      detail: persistence.warning,
      tone: "warn",
    };
  }
  if (!persistence?.enabled) {
    return {
      title: "Local session only",
      detail: "History is available while this app session is open.",
      tone: "idle",
    };
  }
  if (persistence.restored && Number(persistence.itemsRestored) > 0) {
    return {
      title: `${persistence.itemsRestored} restored`,
      detail: "Sanitized transaction history was restored from local trusted state.",
      tone: "good",
    };
  }
  if (persistence.lastSavedAtUnixMs) {
    return {
      title: "Saved",
      detail: `Last saved ${formatTime(persistence.lastSavedAtUnixMs)}.`,
      tone: "good",
    };
  }
  return {
    title: "Ready",
    detail: "Sanitized activity can be saved locally.",
    tone: "good",
  };
}

function activityNextActionSummary(latest, receipt) {
  if (latest?.guidance) {
    return {
      title: userActionLabel(latest.guidance.primaryAction, "Review result"),
      detail: latest.guidance.nextStep ?? latest.guidance.message ?? "Open the transaction details below.",
      tone: latest.guidance.tone ?? activityOutcomeTone(latest),
    };
  }
  if (receipt.tone === "warn") {
    return {
      title: "Wait for network",
      detail: receipt.detail,
      tone: "warn",
    };
  }
  if (latest && activityOutcomeTone(latest) === "good") {
    return {
      title: "Refresh app",
      detail: "Return to the dApp and refresh its state if the result is not visible yet.",
      tone: "good",
    };
  }
  if (latest && activityOutcomeTone(latest) === "bad") {
    return {
      title: "Inspect failure",
      detail: "Use the failure message below before retrying from the dApp.",
      tone: "bad",
    };
  }
  return {
    title: "Use an app",
    detail: "Approve a transaction to start tracking it here.",
    tone: "warn",
  };
}

async function refreshStatus() {
  const response = await invokeCommand("framkey_status");
  if (response?.result) {
    renderStatus(response.result);
    refreshRpcHealth(false).catch(() => {});
    startTrustedAutosmoke(response.result);
  }
  await refreshConnectedSites(false);
  await refreshProviderEvents(false);
  await refreshReviewQueue(false);
  await refreshTransactionActivity(false, false);
  await refreshDappSession(false);
}

async function switchNetwork() {
  const targetChainId = networkSelect.value;
  if (!targetChainId || sameChainId(targetChainId, latestStatus?.chainId)) {
    updateNetworkSwitchState();
    return;
  }
  const response = await invokeCommand("framkey_switch_session_chain", {
    request: { chainId: targetChainId },
  });
  if (response?.result?.status) {
    renderStatus(response.result.status);
  } else {
    await refreshStatus();
  }
  await refreshPortfolio(false).catch(() => {});
  await refreshRpcHealth(false).catch(() => {});
  await refreshTransactionActivity(false, false).catch(() => {});
}

function setWalletConnectionPending(state) {
  walletConnectionPending = state;
  renderWalletProductOverview();
}

function setWalletConnectionError(error) {
  walletConnectionError = error;
  renderWalletProductOverview();
}

function nextWalletConnectionOperation(state) {
  walletConnectionOperationId += 1;
  setWalletConnectionPending(state);
  return walletConnectionOperationId;
}

function isCurrentWalletConnectionOperation(operationId) {
  return operationId === walletConnectionOperationId;
}

function walletConnectionErrorReadiness(error) {
  const message = walletConnectionErrorMessage(error);
  if (isSignerHelperTimeout(message)) {
    return "Signing timeout";
  }
  if (isBiometryLockedOut(message)) {
    return "macOS auth locked";
  }
  if (message.includes("GBxCart") || message.includes("serial") || message.includes("card")) {
    return "Card issue";
  }
  if (message.includes("signer helper")) {
    return "Signing issue";
  }
  return "Connect failed";
}

function walletConnectionErrorText(error) {
  const message = walletConnectionErrorMessage(error);
  if (isSignerHelperTimeout(message)) {
    return "macOS is waiting for a Keychain or local authentication dialog. Complete the system prompt, then retry.";
  }
  if (isBiometryLockedOut(message)) {
    return "macOS local authentication is locked. Use the password prompt or wait before retrying.";
  }
  return truncateWalletConnectionText(message || "Connect failed", 120);
}

function walletConnectionErrorMessage(error) {
  return String(error?.message ?? error?.error?.message ?? error ?? "");
}

function isBiometryLockedOut(message) {
  return message.includes("Biometry is locked out") || message.includes("code -8");
}

function isSignerHelperTimeout(message) {
  return message.includes("signer helper timed out after 45000 ms");
}

function truncateWalletConnectionText(value, maxLength) {
  if (value.length <= maxLength) {
    return value;
  }
  return `${value.slice(0, Math.max(0, maxLength - 3))}...`;
}

async function connectCard() {
  if (walletConnectionPending) {
    return null;
  }
  const existingAddress = latestPortfolio?.address ?? latestAccount?.address ?? null;
  if (existingAddress) {
    renderWalletProductOverview();
    return latestAccount ?? { address: existingAddress };
  }
  const operationId = nextWalletConnectionOperation("connecting");
  walletConnectionError = null;
  accountAddress.textContent = "Reading vault from card; macOS unlock follows";
  accountBalance.textContent = "-";
  setBusy("framkey_getAccount");
  try {
    const response = await providerRequestQuiet("framkey_getAccount");
    if (!isCurrentWalletConnectionOperation(operationId)) {
      return response;
    }
    renderEnvelope(response);
    if (response?.result) {
      walletConnectionError = null;
      renderAccount(response.result);
      refreshPortfolio(false).catch(() => {});
    } else if (response?.error) {
      setWalletConnectionError(response.error);
    }
    return response;
  } catch (error) {
    if (isCurrentWalletConnectionOperation(operationId)) {
      setWalletConnectionError(error);
      renderError(error);
    }
    throw error;
  } finally {
    if (isCurrentWalletConnectionOperation(operationId)) {
      setWalletConnectionPending(null);
    }
  }
}

async function authorizeKeychainHelper() {
  if (keychainHelperAccessPending || walletConnectionPending) {
    return null;
  }
  keychainHelperAccessPending = true;
  walletConnectionError = null;
  renderWalletProductOverview();
  signerHelper.textContent = "Repairing signing access";
  try {
    const response = await invokeCommand("framkey_authorize_keychain_helper");
    if (response?.result) {
      signerHelper.textContent = formatKeychainHelperAccessResult(response.result);
      walletConnectionError = null;
    } else if (response?.error) {
      setWalletConnectionError(response.error);
    }
    return response;
  } catch (error) {
    setWalletConnectionError(error);
    throw error;
  } finally {
    keychainHelperAccessPending = false;
    renderWalletProductOverview();
  }
}

async function disconnectWallet() {
  if (walletConnectionPending) {
    return null;
  }
  const operationId = nextWalletConnectionOperation("disconnecting");
  walletConnectionError = null;
  setBusy("framkey_disconnect_account");
  try {
    const disconnected = await invokeQuiet("framkey_disconnect_account").catch(() => null);
    if (!isCurrentWalletConnectionOperation(operationId)) {
      return disconnected;
    }
    if (disconnected) {
      renderEnvelope(disconnected);
    }
    if (!disconnected?.result) {
      const origins = [...latestConnectedOrigins];
      for (const origin of origins) {
        await invokeQuiet("framkey_revoke_account_permission", { origin });
      }
      await invokeQuiet("framkey_clear_review_queue").catch(() => {});
    }
    latestAccount = null;
    walletConnectionError = null;
    renderPortfolioBaseline();
    clearTokenSendSelection();
    accountAddress.textContent = "Not connected";
    accountBalance.textContent = "-";
    await refreshConnectedSites(false).catch(() => {});
    await refreshReviewQueue(false).catch(() => {});
    renderSessionReadiness();
    renderWalletProductOverview();
    return disconnected;
  } finally {
    if (isCurrentWalletConnectionOperation(operationId)) {
      setWalletConnectionPending(null);
    }
  }
}

async function toggleWalletConnection() {
  if (walletConnectionPending) {
    return null;
  }
  const address = latestPortfolio?.address ?? latestAccount?.address ?? null;
  if (address) {
    await disconnectWallet();
  } else {
    await connectCard();
  }
}

async function refreshAccountBalance(address) {
  if (!address) {
    accountBalance.textContent = "-";
    return;
  }
  accountBalance.textContent = "Loading...";
  try {
    const response = await providerRequestQuiet("eth_getBalance", [address, "latest"]);
    if (typeof response?.result === "string") {
      accountBalance.textContent = formatNativeBalance(response.result);
      return;
    }
    accountBalance.textContent = "unavailable";
  } catch {
    accountBalance.textContent = "unavailable";
  }
}

async function refreshRpcHealth(showOutput = true) {
  setRpcHealthLoading();
  try {
    const response = showOutput
      ? await invokeCommand("framkey_rpc_health")
      : await invokeQuiet("framkey_rpc_health");
    if (response?.result) {
      renderRpcHealth(response.result);
    } else if (response?.error) {
      renderRpcHealthError(response.error);
    }
    return response;
  } catch (error) {
    renderRpcHealthError(error);
    if (showOutput) {
      renderError(error);
    }
    throw error;
  }
}

async function refreshPortfolio(showOutput = true) {
  const address = latestPortfolio?.address ?? latestAccount?.address ?? null;
  if (!address) {
    renderPortfolioBaseline();
    const response = {
      error: {
        code: 4100,
        message: "Connect the vault before refreshing assets",
      },
    };
    if (showOutput) {
      renderEnvelope(response);
    }
    return response;
  }
  setPortfolioLoading();
  try {
    const response = showOutput
      ? await invokeCommand("framkey_wallet_assets")
      : await invokeQuiet("framkey_wallet_assets");
    if (response?.result) {
      renderPortfolio(response.result);
    } else if (response?.error) {
      renderPortfolioError(response.error);
    }
    return response;
  } catch (error) {
    renderPortfolioError(error);
    if (showOutput) {
      renderError(error);
    }
    throw error;
  }
}

async function sendNativeTransfer(event) {
  event?.preventDefault();
  const request = {
    to: nativeSendTo.value.trim(),
    amount: nativeSendAmount.value.trim(),
    chainId: latestStatus?.chainId,
  };
  setNativeSendState("Review", "busy", "Waiting for trusted approval");
  nativeSendSubmitButton.disabled = true;
  try {
    const response = await invokeCommand("framkey_send_native_transfer", { request });
    if (response?.result) {
      renderNativeSendResult(response.result);
      await refreshReviewQueue(false);
      await refreshTransactionActivity(false, false);
      await refreshPortfolio(false).catch(() => {});
      setActiveWorkspace("activity");
      return response;
    }
    if (response?.error) {
      renderNativeSendError(response.error);
    }
    return response;
  } catch (error) {
    renderNativeSendError(error);
    throw error;
  } finally {
    nativeSendSubmitButton.disabled = false;
  }
}

async function sendTokenTransfer(event) {
  event?.preventDefault();
  if (!selectedTokenForSend) {
    renderTokenSendError(new Error("Select a token first"));
    return null;
  }
  const request = {
    tokenContract: selectedTokenForSend.contractAddress,
    to: tokenSendTo.value.trim(),
    amount: tokenSendAmount.value.trim(),
    decimals: selectedTokenForSend.decimals,
    symbol: selectedTokenForSend.symbol,
    chainId: selectedTokenForSend.chainId ?? latestStatus?.chainId,
  };
  setTokenSendState("Review", "busy", "Waiting for trusted approval");
  tokenSendSubmitButton.disabled = true;
  try {
    const response = await invokeCommand("framkey_send_token_transfer", { request });
    if (response?.result) {
      renderTokenSendResult(response.result);
      await refreshReviewQueue(false);
      await refreshTransactionActivity(false, false);
      await refreshPortfolio(false).catch(() => {});
      setActiveWorkspace("activity");
      return response;
    }
    if (response?.error) {
      renderTokenSendError(response.error);
    }
    return response;
  } catch (error) {
    renderTokenSendError(error);
    throw error;
  } finally {
    tokenSendSubmitButton.disabled = !selectedTokenForSend;
  }
}

async function refreshTransactionActivity(showOutput = true, refreshReceipts = false) {
  if (refreshReceipts) {
    clearReceiptAutoRefreshTimer();
    receiptAutoRefreshInFlight = true;
    renderReceiptTrackingState();
    renderProductOverview();
  }
  try {
    const response = await invokeQuiet("framkey_transaction_activity", {
      request: { refreshReceipts },
    });
    if (refreshReceipts) {
      lastReceiptRefreshAtUnixMs = Date.now();
      latestReceiptRefreshError = null;
    }
    if (response?.result) {
      renderTransactionActivity(response.result);
    }
    if (showOutput) {
      renderEnvelope(response);
    }
    return response;
  } catch (error) {
    if (showOutput) {
      renderError(error);
    }
    if (refreshReceipts) {
      latestReceiptRefreshError = error?.message ?? String(error);
    }
    throw error;
  } finally {
    if (refreshReceipts) {
      receiptAutoRefreshInFlight = false;
      scheduleReceiptAutoRefresh(latestTransactionActivity);
      renderReceiptTrackingState();
      renderProductOverview();
    }
  }
}

async function openDapp(url = null, label = null) {
  activeDappTarget = label ?? dappTargetLabel(url);
  renderSessionReadiness();
  const response = await invokeCommand("open_dapp_webview", { url });
  if (response?.result?.state) {
    renderDappSession(response.result.state);
  } else {
    await refreshDappSession(false).catch(() => {});
  }
  renderSessionReadiness();
}

async function refreshDappSession(showOutput = true) {
  try {
    const response = showOutput
      ? await invokeCommand("framkey_dapp_session")
      : await invokeQuiet("framkey_dapp_session");
    if (response?.result) {
      renderDappSession(response.result);
    }
    return response;
  } catch (error) {
    if (showOutput) {
      renderError(error);
    }
    throw error;
  }
}

async function navigateDapp(action) {
  const response = await invokeCommand("framkey_navigate_dapp", {
    request: { action },
  });
  if (response?.result?.state) {
    renderDappSession(response.result.state);
  } else {
    await refreshDappSession(false).catch(() => {});
  }
}

async function runCompatibilityCheck(target) {
  activeCompatibilityChecks.add(target.key);
  renderCompatibilityStatus();
  try {
    if (target.key === "uniswap") {
      dappUrl.value = "https://app.uniswap.org/";
    }
    if (target.key === "aave") {
      dappUrl.value = "https://app.aave.com/";
    }
    await openDapp(target.openUrl, target.label);
    await delay(COMPATIBILITY_CHECK_SETTLE_MS);
    const response = await invokeCommand("framkey_run_dapp_compatibility_check", {
      request: { mode: "read" },
    });
    await delay(COMPATIBILITY_CHECK_REFRESH_MS);
    await refreshProviderEvents(false).catch(() => {});
    await refreshReviewQueue(false).catch(() => {});
    await refreshConnectedSites(false).catch(() => {});
    renderCompatibilityStatus();
    return response;
  } finally {
    activeCompatibilityChecks.delete(target.key);
    renderCompatibilityStatus();
  }
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function createVault() {
  if (creatingVault) {
    return;
  }
  const generation = Number.parseInt(vaultGeneration.value, 10);
  if (!Number.isSafeInteger(generation) || generation < 1) {
    renderError(new Error("Generation must be a positive integer"));
    setCreateVaultStatus("error", { message: "Enter a positive backup version." });
    return;
  }
  if (!confirmOverwrite.checked) {
    renderError(new Error("Confirm the configured vault device write before creating the vault"));
    setCreateVaultStatus("error", { message: "Confirm the connected vault device replacement first." });
    updateCreateVaultActionState();
    return;
  }
  createVaultCompleted = false;
  setCreateVaultBusy(true);
  try {
    const response = await invokeCommand("framkey_create_keychain_vault", {
      request: {
        generation,
        recoveryOutDir: recoveryOutDir.value,
        confirmOverwrite: confirmOverwrite.checked,
      },
    });
    if (response?.error) {
      throw new Error(response.error.message ?? "Create failed");
    }
    if (!response?.result) {
      throw new Error("Create did not return a recovery pack");
    }
    renderRecoveryOutcome(response.result);
    const recommended = recommendedRecoveryFileBuckets(response.result);
    if (recoveryFilesFromBuckets(recommended).length > 0) {
      setRecoveryFileBuckets(recommended, { resetOutcomes: false });
    }
    renderRecoveryInputStatus();
    createVaultCompleted = true;
    setCreateVaultStatus("success", { result: response.result });
  } catch (error) {
    setCreateVaultStatus("error", {
      message: operationErrorMessage(error),
    });
    throw error;
  } finally {
    setCreateVaultBusy(false);
  }
}

function updateCreateVaultActionState() {
  const completed = createVaultCompleted && !creatingVault;
  if (createConfirmRow) {
    createConfirmRow.hidden = completed;
  }
  createVaultButton.hidden = completed;
  const confirmed = Boolean(confirmOverwrite.checked);
  createVaultButton.disabled = completed || creatingVault || !confirmed;
  if (creatingVault) {
    createVaultButton.textContent = "Creating...";
  } else if (completed) {
    createVaultButton.textContent = "Wallet written";
  } else {
    createVaultButton.textContent = confirmed ? "Create wallet and write GBA" : "Confirm replacement";
  }
}

function setCreateVaultBusy(busy) {
  creatingVault = busy;
  vaultGeneration.disabled = busy;
  recoveryOutDir.disabled = busy;
  confirmOverwrite.disabled = busy;
  chooseRecoveryOutDirButton.disabled = busy;
  updateCreateVaultActionState();
  if (busy) {
    setCreateVaultStatus("busy");
  }
}

function setCreateVaultStatus(state, options = {}) {
  createVaultStatus.hidden = false;
  createVaultStatus.dataset.tone = state;
  createVaultStatus.replaceChildren();

  if (state === "busy") {
    createVaultWriteNote.textContent = "Keep the GBA cart connected until this finishes.";
    createVaultStatus.append(
      operationStatusHeader("Preparing the Mac key and writing the GBA card"),
      operationProgressBar(),
      textSpan("If macOS asks for authorization, approve it. FRAMKey will keep working and write the connected card."),
    );
    return;
  }

  if (state === "success") {
    createVaultWriteNote.textContent = "GBA write complete. Move these backup files now.";
    createVaultStatus.append(
      operationStatusHeader("Wallet created and GBA written"),
      createBackupDestinationSummary(options.result),
    );
    return;
  }

  if (state === "error") {
    createVaultCompleted = false;
    updateCreateVaultActionState();
    createVaultWriteNote.textContent = "The new wallet was not completed.";
    createVaultStatus.append(
      operationStatusHeader("Create failed"),
      textSpan(options.message ?? "Check the error details and try again."),
    );
    return;
  }

  createVaultWriteNote.textContent = "Confirm replacement before writing";
  createVaultStatus.hidden = true;
}

function resetCreateCompletion() {
  if (!createVaultCompleted) {
    return;
  }
  createVaultCompleted = false;
  setCreateVaultStatus("idle");
  updateCreateVaultActionState();
}

function operationStatusHeader(text) {
  const header = document.createElement("div");
  header.className = "operation-status-header";
  const spinner = document.createElement("span");
  spinner.className = "operation-spinner";
  spinner.setAttribute("aria-hidden", "true");
  const title = document.createElement("strong");
  title.textContent = text;
  header.append(spinner, title);
  return header;
}

function operationProgressBar() {
  const progress = document.createElement("div");
  progress.className = "operation-progress";
  progress.append(document.createElement("span"));
  return progress;
}

function operationErrorMessage(error) {
  const message = error?.message ?? String(error);
  if (
    message.includes("Security.framework status -34018") ||
    message.includes("Keychain entitlements") ||
    message.includes("errSecMissingEntitlement") ||
    message.includes("restricted entitlements")
  ) {
    return "macOS blocked this Keychain operation. The wallet was not written. Restart FRAMKey after rebuilding the current app, then try again.";
  }
  if (message.includes("exited with signal: 9") || message.includes("before returning JSON")) {
    return "The signing service could not start under macOS code-signing rules. Rebuild and restart FRAMKey, then try again.";
  }
  return message;
}

function createBackupDestinationSummary(result) {
  const backups = result?.recoveryBackups ?? {};
  const files = backups.files ?? [];
  const section = document.createElement("div");
  section.className = "create-backup-summary";

  const materials = document.createElement("div");
  materials.className = "create-backup-materials";
  for (const item of createBackupMaterialItems(files)) {
    materials.append(createBackupMaterialCard(item));
  }
  section.append(materials, createBackupDetails(backups, files));
  return section;
}

function createBackupMaterialItems(files) {
  const bundles = files.filter((file) => file.kind === "bundle");
  const cloud1 = bundles.find((file) => recoveryArtifactRole(file) === "icloud");
  const cloud2 = bundles.find((file) => recoveryArtifactRole(file) === "google");
  const local1 = bundles.find((file) => file.group === "local_physical");
  const local2 = bundles.find((file) => file.group === "remote_physical");

  return [
    {
      key: "cloud1",
      label: "Cloud 1",
      badge: "iCloud",
      body: "Put this plain backup file in iCloud Drive.",
      file: cloud1,
    },
    {
      key: "cloud2",
      label: "Cloud 2",
      badge: "Google",
      body: "Put this plain backup file in Google Drive.",
      file: cloud2,
    },
    {
      key: "local1",
      label: "Local 1",
      badge: "Physical",
      body: "Copy this plain backup file to local storage.",
      file: local1,
    },
    {
      key: "local2",
      label: "Local 2",
      badge: "Off-site",
      body: "Store this away from this Mac and GBA card.",
      file: local2,
    },
  ];
}

function createBackupMaterialCard(item) {
  const card = document.createElement("article");
  card.className = "create-backup-material";
  card.dataset.kind = item.key;

  const title = document.createElement("div");
  title.className = "create-backup-material-title";
  const label = document.createElement("strong");
  label.textContent = item.label;
  const badge = document.createElement("span");
  badge.textContent = item.badge;
  title.append(label, badge);

  const body = document.createElement("p");
  body.textContent = item.body;

  const action = document.createElement("button");
  action.type = "button";
  action.className = "create-backup-material-action";
  action.textContent = item.file?.path ? "Show file" : "Missing";
  action.disabled = !item.file?.path;
  action.addEventListener("pointerdown", (event) => {
    event.stopPropagation();
  });
  action.addEventListener("click", (event) => {
    event.preventDefault();
    event.stopPropagation();
    revealPath(item.file?.path).catch(() => {});
  });

  card.append(title, body, action);
  return card;
}

function createBackupDetails(backups, files) {
  const details = document.createElement("details");
  details.className = "create-backup-details";

  const summary = document.createElement("summary");
  const label = document.createElement("strong");
  label.textContent = "File details";
  const count = document.createElement("span");
  count.textContent = `${files.length} files`;
  summary.append(label, count);

  const folder = document.createElement("div");
  folder.className = "create-backup-detail-folder";
  const folderText = document.createElement("span");
  folderText.textContent = backups.outDir ?? "Created in the selected backup folder";
  const folderButton = document.createElement("button");
  folderButton.type = "button";
  folderButton.textContent = "Open folder";
  folderButton.disabled = !backups.outDir;
  folderButton.addEventListener("click", (event) => {
    event.preventDefault();
    event.stopPropagation();
    revealPath(backups.outDir).catch(() => {});
  });
  folder.append(folderText, folderButton);

  const list = document.createElement("ul");
  list.className = "create-backup-detail-list";
  for (const file of files) {
    const item = document.createElement("li");
    const copy = document.createElement("div");
    const name = document.createElement("strong");
    name.textContent = fileNameFromPath(file.path) || file.member || recoveryArtifactKindLabel(file);
    const meta = document.createElement("span");
    meta.textContent = [recoveryArtifactKindLabel(file), file.member, shortHash(file.blake3)]
      .filter(Boolean)
      .join(" · ");
    copy.append(name, meta);

    const reveal = document.createElement("button");
    reveal.type = "button";
    reveal.textContent = "Show";
    reveal.disabled = !file.path;
    reveal.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      revealPath(file.path).catch(() => {});
    });

    item.append(copy, reveal);
    list.append(item);
  }

  details.append(summary, folder, list);
  return details;
}

async function recoverVault() {
  if (recoveringVault) {
    return;
  }
  const state = recoveryInputState();
  const files = state.files;
  if (files.length === 0) {
    renderError(new Error("At least one backup file path is required"));
    renderRecoveryInputStatus();
    return;
  }
  if (!state.canAttemptRestore) {
    renderError(new Error(state.policy));
    renderRecoveryInputStatus();
    return;
  }
  const backupPath = recoveryVaultSourcePath(files);
  vaultBackupPath.value = backupPath;
  setRecoverVaultBusy(true);
  try {
    const response = await invokeCommand("framkey_recover_keychain_vault", {
      request: {
        vaultBackupPath: backupPath,
        recoveryFiles: files,
        confirmOverwrite: recoverOverwrite.checked,
      },
    });
    if (response?.error) {
      throw new Error(response.error.message ?? "Restore failed");
    }
    if (!response?.result) {
      throw new Error("Restore did not return a vault image");
    }
    renderRecoveryOutcome(response.result);
    renderRecoveryInputStatus();
    setRestoreWriteStatus("success", { message: "Restored wallet written to the connected vault device." });
  } catch (error) {
    setRestoreWriteStatus("error", {
      message: operationErrorMessage(error),
    });
    throw error;
  } finally {
    setRecoverVaultBusy(false);
  }
}

function setRecoverVaultBusy(busy) {
  recoveringVault = busy;
  recoverOverwrite.disabled = busy;
  recoverVaultButton.disabled = busy || !recoverVaultReady();
  recoverVaultButton.textContent = busy ? "Restoring..." : "Restore wallet";
  if (busy) {
    setRestoreWriteStatus("busy");
  }
}

function setRestoreWriteStatus(state, options = {}) {
  restoreWriteDetail.hidden = false;
  restoreWriteDetail.dataset.tone = state;
  restoreWriteDetail.replaceChildren();

  if (state === "busy") {
    restoreWriteSummary.textContent = "Writing restored wallet to the connected vault device";
    restoreWriteDetail.append(
      operationStatusHeader("Rebinding the Mac key and writing the GBA card"),
      operationProgressBar(),
      textSpan("Keep the GBA cart connected until the restore button is enabled again."),
    );
    return;
  }

  if (state === "success") {
    restoreWriteSummary.textContent = "Restored wallet written";
    restoreWriteDetail.append(operationStatusHeader("Restore complete"), textSpan(options.message));
    return;
  }

  if (state === "error") {
    restoreWriteSummary.textContent = "Restore failed";
    restoreWriteDetail.append(
      operationStatusHeader("Restore failed"),
      textSpan(options.message ?? "Check the error details and try again."),
    );
    return;
  }

  restoreWriteDetail.hidden = true;
}

function recoverVaultReady() {
  const state = recoveryInputState();
  return state.canAttemptRestore && recoverOverwrite.checked;
}

async function chooseRecoveryOutDir() {
  const response = await invokeCommand("framkey_pick_recovery_out_dir");
  const path = response?.result?.paths?.[0];
  if (path && !response.result.cancelled) {
    recoveryOutDir.value = path;
    resetCreateCompletion();
  }
}

async function chooseRecoveryFileForSlot(slotKey) {
  const response = await invokeCommand("framkey_pick_recovery_files");
  const paths = response?.result?.paths ?? [];
  if (paths.length > 0 && !response.result.cancelled) {
    assignRecoveryFileToSlot(slotKey, paths);
    renderRecoveryInputStatus();
  }
}

async function refreshRecoveryState(showOutput = true) {
  const response = showOutput
    ? await invokeCommand("framkey_recovery_state")
    : await invokeQuiet("framkey_recovery_state");
  if (response?.result) {
    renderRecoveryState(response.result);
    prefillRecoveryFilesFromBackup();
    renderRecoveryInputStatus();
    reportRecoveryStateRestoreSmoke(response.result).catch(() => {});
  }
  return response;
}

async function clearRecoveryPlan() {
  clearBackupPlacementStateForCurrentPlan();
  const response = await invokeCommand("framkey_clear_recovery_state");
  if (response?.result) {
    renderRecoveryState(response.result);
    clearRecoveryFileBuckets({ resetOutcomes: false });
    renderRecoveryInputStatus();
  }
}

async function refreshReviewQueue(showOutput = true) {
  try {
    const response = await invokeQuiet("framkey_review_queue");
    if (response?.result) {
      renderReviewQueue(response.result);
    }
    if (showOutput) {
      renderEnvelope(response);
    }
    return response;
  } catch (error) {
    if (showOutput) {
      renderError(error);
    }
    throw error;
  }
}

async function refreshConnectedSites(showOutput = true) {
  try {
    const response = await invokeQuiet("framkey_account_permissions");
    if (response?.result) {
      renderConnectedSites(response.result);
    }
    if (showOutput) {
      renderEnvelope(response);
    }
    return response;
  } catch (error) {
    if (showOutput) {
      renderError(error);
    }
    throw error;
  }
}

async function refreshProviderEvents(showOutput = true) {
  try {
    const response = await invokeQuiet("framkey_provider_events");
    if (response?.result) {
      renderProviderEvents(response.result);
    }
    if (showOutput) {
      renderEnvelope(response);
    }
    return response;
  } catch (error) {
    if (showOutput) {
      renderError(error);
    }
    throw error;
  }
}

async function clearProviderEvents() {
  const response = await invokeCommand("framkey_clear_provider_events");
  if (response?.result) {
    await refreshProviderEvents(false);
  }
}

async function revokeConnectedSite(origin) {
  const response = await invokeCommand("framkey_revoke_account_permission", { origin });
  if (response?.result) {
    await refreshConnectedSites(false);
  }
}

async function smokeReport(stage, detail = {}) {
  const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return;
  }
  try {
    await invoke("framkey_smoke_event", { event: { stage, detail } });
  } catch {
    // Smoke reporting is optional and disabled outside runtime smoke mode.
  }
}

function startTrustedAutosmoke(status) {
  const autosmokeEnabled =
    status?.capabilities?.runtimeSmoke || status?.capabilities?.trustedAutosmoke;
  if (trustedAutosmokeStarted || !autosmokeEnabled) {
    return;
  }
  trustedAutosmokeStarted = true;
  if (!status.wallet?.mock) {
    smokeReport("trusted_ui_autosmoke_skipped", {
      reason: "wallet_not_mock",
      wallet: status.wallet?.kind,
    });
    return;
  }
  smokeReport("trusted_ui_autosmoke_started", {
    wallet: status.wallet?.kind,
    simulation: status.simulation?.kind,
    runtimeSmoke: Boolean(status.capabilities?.runtimeSmoke),
    trustedAutosmoke: Boolean(status.capabilities?.trustedAutosmoke),
  });
  smokeReport("trusted_ui_helper_status_smoke", {
    ready: Boolean(status.signerHelper?.ready),
    readiness: status.signerHelper?.readiness,
    location: status.signerHelper?.location,
    sandbox: status.signerHelper?.sandbox,
    hashPinned: Boolean(status.signerHelper?.hashPinned),
    hashMatches: status.signerHelper?.hashMatches,
  });
  refreshDappSession(false)
    .then((response) => {
      smokeReport("trusted_ui_dapp_session_smoke", {
        ok: Boolean(response?.result),
        targetLabel: response?.result?.targetLabel,
        origin: response?.result?.origin,
        loadStatus: response?.result?.loadStatus,
        queryExposed: String(response?.result?.currentUrl ?? "").includes("?"),
        fragmentExposed: String(response?.result?.currentUrl ?? "").includes("#"),
      });
    })
    .catch((error) => {
      smokeReport("trusted_ui_dapp_session_smoke_error", {
        message: error?.message ?? String(error),
      });
    });
  refreshPortfolio(false)
    .then((response) => {
      smokeReport("trusted_ui_portfolio_smoke", {
        ok: Boolean(response?.result),
        rpc: Boolean(response?.result?.rpc),
        tokenCount: response?.result?.tokens?.length ?? 0,
        errors: response?.result?.errors?.length ?? (response?.error ? 1 : 0),
      });
    })
    .catch((error) => {
      smokeReport("trusted_ui_portfolio_smoke_error", {
        message: error?.message ?? String(error),
      });
    });
  refreshRpcHealth(false)
    .then((response) => {
      smokeReport("trusted_ui_rpc_health_smoke", {
        ok: Boolean(response?.result),
        healthy: Boolean(response?.result?.healthy),
        status: response?.result?.status,
        chainMatches: Boolean(response?.result?.chainMatches),
        latestBlock: Boolean(response?.result?.latestBlock),
        tokenExposed: Boolean(response?.result?.tokenExposed),
        rpcUrlExposed: Boolean(response?.result?.rpcUrlExposed),
      });
    })
    .catch((error) => {
      smokeReport("trusted_ui_rpc_health_smoke_error", {
        message: error?.message ?? String(error),
      });
    });
  refreshRecoveryState(false)
    .then((response) => {
      smokeReport("trusted_ui_recovery_state_smoke", {
        ok: Boolean(response?.result),
        restored: Boolean(response?.result?.persistence?.restored),
        backup: Boolean(response?.result?.backupOutcome),
        drill: Boolean(response?.result?.drillOutcome),
        recover: Boolean(response?.result?.recoverOutcome),
        shareFileCount: response?.result?.backupOutcome?.recoveryBackups?.shareFileCount,
      });
    })
    .catch((error) => {
      smokeReport("trusted_ui_recovery_state_smoke_error", {
        message: error?.message ?? String(error),
      });
    });
  runRecoveryAutosmoke(status).catch((error) => {
    smokeReport("trusted_ui_recovery_smoke_error", {
      message: error?.message ?? String(error),
    });
  });
  runCompatibilityAutosmoke().catch((error) => {
    smokeReport("trusted_ui_compatibility_check_smoke", {
      ok: false,
      message: error?.message ?? String(error),
    });
  });
  runWalletSendAutosmoke(status).catch((error) => {
    smokeReport("trusted_ui_wallet_send_smoke_error", {
      message: error?.message ?? String(error),
    });
  });
  const timer = setInterval(() => {
    trustedAutosmokeTick().catch((error) => {
      smokeReport("trusted_ui_autosmoke_error", {
        message: error?.message ?? String(error),
      });
    });
  }, 500);
  const configuredDurationMs = Number(status.capabilities?.trustedAutosmokeDurationMs);
  const durationMs =
    Number.isFinite(configuredDurationMs) && configuredDurationMs > 0
      ? configuredDurationMs
      : status.capabilities?.runtimeSmoke
        ? 20_000
        : 45_000;
  setTimeout(() => {
    clearInterval(timer);
    smokeReport("trusted_ui_autosmoke_stopped", { durationMs });
  }, durationMs);
}

async function runCompatibilityAutosmoke() {
  await delay(COMPATIBILITY_CHECK_SETTLE_MS);
  const response = await invokeQuiet("framkey_run_dapp_compatibility_check", {
    request: { mode: "read" },
  });
  await smokeReport("trusted_ui_compatibility_check_smoke", {
    ok: Boolean(response?.result?.started),
    mode: response?.result?.mode,
    readOnly: Boolean(response?.result?.readOnly),
    errorCode: response?.error?.code,
    errorMessage: response?.error?.message,
  });
  await delay(COMPATIBILITY_CHECK_REFRESH_MS);
  await refreshProviderEvents(false).catch(() => {});
}

async function runRecoveryAutosmoke(status) {
  if (recoveryAutosmokeStarted || !status?.capabilities?.recoveryAutosmoke) {
    return;
  }
  recoveryAutosmokeStarted = true;
  if (!status.wallet?.mock) {
    await smokeReport("trusted_ui_recovery_smoke_skipped", {
      reason: "wallet_not_mock",
      wallet: status.wallet?.kind,
    });
    return;
  }
  const response = await invokeQuiet("framkey_recovery_smoke_pack", {
    request: {},
  });
  if (response?.result) {
    renderRecoveryOutcome(response.result);
    await smokeReport("trusted_ui_recovery_smoke", {
      ok: true,
      outDir: response.result.outDir,
      shareFileCount: response.result.recoveryBackups?.shareFileCount,
      cloudOnlyCanRecover: response.result.cloudOnlyDrill?.canRecover,
      recommendedCanRecover: response.result.recommendedDrill?.canRecover,
      walletSecretTouched: response.result.walletSecretTouched,
      recoveryShareBytesPrinted: response.result.recoveryShareBytesPrinted,
    });
  } else {
    await smokeReport("trusted_ui_recovery_smoke", {
      ok: false,
      errorCode: response?.error?.code,
      errorMessage: response?.error?.message,
    });
  }
}

async function runWalletSendAutosmoke(status) {
  if (walletSendAutosmokeStarted || !status?.capabilities?.walletSendAutosmoke) {
    return;
  }
  walletSendAutosmokeStarted = true;
  if (!status.wallet?.mock) {
    await smokeReport("trusted_ui_wallet_send_smoke_skipped", {
      reason: "wallet_not_mock",
      wallet: status.wallet?.kind,
    });
    return;
  }
  await smokeReport("trusted_ui_wallet_send_smoke_started", {
    chainId: status.chainId,
    nativeSend: status.capabilities?.nativeSend,
    tokenSend: status.capabilities?.tokenSend,
  });

  await delay(4_000);
  await runNativeSendFormAutosmoke();
  await runTokenSendFormAutosmoke();
}

async function runNativeSendFormAutosmoke() {
  nativeSendTo.value = WALLET_SEND_AUTOSMOKE_RECIPIENT;
  nativeSendAmount.value = WALLET_SEND_AUTOSMOKE_NATIVE_AMOUNT;
  const response = await sendNativeTransfer({ preventDefault() {} });
  await smokeReport("trusted_ui_native_send_form_smoke", {
    ok: Boolean(response?.result),
    status: response?.result?.status,
    operation: response?.result?.operation,
    transactionHashPresent: Boolean(response?.result?.transactionHash),
    errorCode: response?.error?.code ?? response?.result?.providerError?.code,
    errorMessage: response?.error?.message ?? response?.result?.providerError?.message,
  });
}

async function runTokenSendFormAutosmoke() {
  const token = await waitForSendablePortfolioToken(15_000);
  if (!token) {
    await smokeReport("trusted_ui_token_send_form_smoke", {
      ok: false,
      skipped: true,
      reason: "no_sendable_token",
    });
    return;
  }
  selectTokenForSend(token);
  tokenSendTo.value = WALLET_SEND_AUTOSMOKE_RECIPIENT;
  tokenSendAmount.value = WALLET_SEND_AUTOSMOKE_TOKEN_AMOUNT;
  const response = await sendTokenTransfer({ preventDefault() {} });
  await smokeReport("trusted_ui_token_send_form_smoke", {
    ok: Boolean(response?.result),
    status: response?.result?.status,
    operation: response?.result?.operation,
    symbol: response?.result?.symbol ?? selectedTokenForSend?.symbol,
    transactionHashPresent: Boolean(response?.result?.transactionHash),
    errorCode: response?.error?.code ?? response?.result?.providerError?.code,
    errorMessage: response?.error?.message ?? response?.result?.providerError?.message,
  });
}

async function waitForSendablePortfolioToken(timeoutMs) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    const response = await refreshPortfolio(false).catch(() => null);
    const tokens = response?.result?.tokens ?? latestPortfolio?.tokens ?? [];
    const token = tokens.find((item) => canSendPortfolioToken(item));
    if (token) {
      return token;
    }
    await delay(750);
  }
  return null;
}

async function trustedAutosmokeTick() {
  const response = await refreshReviewQueue(false);
  const requests = response?.result?.requests ?? [];
  for (const request of requests) {
    if (request.status !== "pending") {
      continue;
    }
    const action = approveActionForRequest(request);
    if (action.disabled) {
      await smokeReport("trusted_ui_autosmoke_blocked", {
        reviewId: request.id,
        method: request.method,
        kind: request.kind,
      });
      continue;
    }
    await smokeReport("trusted_ui_autosmoke_approving", {
      reviewId: request.id,
      method: request.method,
      kind: request.kind,
      decision: action.decision,
      assetStatus: request.summary?.assetContext?.status,
      assetTokenCount: request.summary?.assetContext?.tokens?.length,
      typedIntent: request.summary?.typedData?.intent,
    });
    await decideReviewRequest(request, action.decision);
  }
}

async function dismissReviewRequest(reviewId) {
  const response = await invokeCommand("framkey_dismiss_review_request", { reviewId });
  if (response?.result) {
    await refreshReviewQueue(false);
  }
}

async function decideReviewRequest(request, decision) {
  const response = await invokeCommand("framkey_decide_review_request", {
    reviewId: request.id,
    decisionToken: request.decisionToken,
    decision,
  });
  if (response?.result) {
    await refreshReviewQueue(false);
    await refreshConnectedSites(false);
    await refreshTransactionActivity(false, false);
    if (request.kind === "watch_asset") {
      setTimeout(() => {
        refreshReviewQueue(false).catch(() => {});
        refreshPortfolio(false)
          .then((portfolioResponse) => {
            smokeReport("trusted_ui_watch_asset_smoke", {
              ok: Boolean(portfolioResponse?.result),
              watched: portfolioResponse?.result?.tokenScan?.watched,
              tokenCount: portfolioResponse?.result?.tokens?.length,
              errorCode: portfolioResponse?.error?.code,
              errorMessage: portfolioResponse?.error?.message,
            });
          })
          .catch((error) => {
            smokeReport("trusted_ui_watch_asset_smoke", {
              ok: false,
              errorMessage: error?.message ?? String(error),
            });
          });
      }, 500);
    }
    if (request.kind === "network_switch") {
      setTimeout(() => {
        refreshStatus().catch(() => {});
        refreshPortfolio(false).catch(() => {});
      }, 500);
    }
  }
}

async function clearReviewQueue() {
  const response = await invokeCommand("framkey_clear_review_queue");
  if (response?.result) {
    await refreshReviewQueue(false);
    await refreshTransactionActivity(false, false);
  }
}

function renderEnvelope(response) {
  if (response?.error) {
    setBridgeState("Error", "bad");
  } else {
    setBridgeState("Ready", "good");
  }
  output.textContent = JSON.stringify(response ?? null, null, 2);
}

function renderStatus(status) {
  latestStatus = status;
  runtimeSummary.textContent = `${status.app ?? "framkey-desktop"} ${status.version ?? ""}`.trim();
  chainId.textContent = status.chainId ?? "-";
  networkName.textContent = formatNetwork(status.network);
  accountAddress.textContent = "Not connected";
  accountBalance.textContent = "-";
  walletMode.textContent = formatWallet(status.wallet);
  rpcStatus.textContent = formatRpc(status.rpc);
  device.textContent = formatDevice(status.device);
  signerHelper.textContent = formatSignerHelper(status.signerHelper);
  nativeSendAmount.placeholder = `0.01 ${nativeSymbolForStatus(status)}`;
  if (selectedTokenForSend?.chainId && status.chainId && !sameChainId(selectedTokenForSend.chainId, status.chainId)) {
    clearTokenSendSelection();
  }
  renderNetworkOptions(status);
  renderCapabilities(status.capabilities ?? {});
  renderSessionReadiness();
  renderProductOverview();
}

function renderDappSession(session) {
  latestDappSession = session;
  const dappOpen = Boolean(session?.open);
  const targetLabel = dappOpen ? (session?.targetLabel ?? "App") : "No app open";
  activeDappTarget = targetLabel;
  if (dappCurrentTarget) {
    dappCurrentTarget.textContent = targetLabel;
  }
  if (dappCurrentOrigin) {
    dappCurrentOrigin.textContent = session?.origin ? shortOrigin(session.origin) : "-";
  }
  if (dappCurrentUrl) {
    const visibleUrl = dappOpen ? (session?.currentUrl ?? session?.requestedUrl ?? "-") : "-";
    dappCurrentUrl.textContent = visibleUrl;
    dappCurrentUrl.title = visibleUrl === "-" ? "" : visibleUrl;
  }
  if (dappLoadStatus) {
    dappLoadStatus.textContent = dappLoadStatusText(session);
    dappLoadStatus.dataset.tone = dappLoadTone(session);
  }
  if (dappNavBackButton) {
    dappNavBackButton.disabled = !dappOpen;
  }
  if (dappNavForwardButton) {
    dappNavForwardButton.disabled = !dappOpen;
  }
  if (dappNavReloadButton) {
    dappNavReloadButton.disabled = !dappOpen;
  }
  if (dappUpdated) {
    dappUpdated.textContent = formatTime(session?.updatedAtUnixMs);
  }
  renderSessionReadiness();
}

function setRpcHealthLoading() {
  rpcHealthSummary.textContent = "Checking";
  rpcHealthSummary.dataset.tone = "busy";
  rpcHealthProvider.textContent = "Alchemy";
  rpcHealthChain.textContent = latestStatus?.chainId ? `expected ${latestStatus.chainId}` : "-";
  rpcHealthBlock.textContent = "-";
  rpcHealthLatency.textContent = "-";
  rpcHealthUpdated.textContent = "-";
  rpcHealthDetail.dataset.tone = "warn";
  rpcHealthDetail.replaceChildren(textSpan("RPC health probe is running"));
}

function renderRpcHealth(health) {
  latestRpcHealth = health;
  const tone = rpcHealthTone(health);
  rpcHealthSummary.textContent = rpcHealthSummaryText(health);
  rpcHealthSummary.dataset.tone = tone;
  rpcHealthProvider.textContent = health.provider === "alchemy" ? "Alchemy" : labelize(health.provider ?? "RPC");
  rpcHealthChain.textContent = rpcHealthChainText(health);
  rpcHealthBlock.textContent = health.latestBlock
    ? `Block ${formatHexInteger(health.latestBlock)}`
    : "-";
  rpcHealthLatency.textContent = Number.isFinite(health.latencyMs) ? `${health.latencyMs}ms` : "-";
  rpcHealthUpdated.textContent = formatTime(health.checkedAtUnixMs);
  rpcHealthDetail.dataset.tone = tone;
  rpcHealthDetail.replaceChildren(textSpan(rpcHealthDetailText(health)));
  rpcStatus.textContent = formatRpcWithHealth(latestStatus?.rpc, health);
  renderSessionReadiness();
}

function renderRpcHealthError(error) {
  latestRpcHealth = {
    healthy: false,
    status: "error",
    error: { message: error?.message ?? error?.error?.message ?? "RPC health unavailable" },
  };
  rpcHealthSummary.textContent = "Unavailable";
  rpcHealthSummary.dataset.tone = "bad";
  rpcHealthProvider.textContent = "Alchemy";
  rpcHealthChain.textContent = "-";
  rpcHealthBlock.textContent = "-";
  rpcHealthLatency.textContent = "-";
  rpcHealthUpdated.textContent = "-";
  rpcHealthDetail.dataset.tone = "bad";
  rpcHealthDetail.replaceChildren(textSpan(latestRpcHealth.error.message));
  renderSessionReadiness();
}

function renderRpcHealthBaseline() {
  latestRpcHealth = null;
  rpcHealthSummary.textContent = "Not checked";
  rpcHealthSummary.dataset.tone = "idle";
  rpcHealthProvider.textContent = "Alchemy";
  rpcHealthChain.textContent = "-";
  rpcHealthBlock.textContent = "-";
  rpcHealthLatency.textContent = "-";
  rpcHealthUpdated.textContent = "-";
  rpcHealthDetail.dataset.tone = "warn";
  rpcHealthDetail.replaceChildren(textSpan("Token and endpoint are hidden"));
}

function renderNetworkOptions(status) {
  const supported = Array.isArray(status.supportedChains) ? status.supportedChains : [];
  const active = status.chainId ?? "";
  networkSelect.replaceChildren();
  let hasActiveOption = false;
  for (const chain of supported) {
    const chainValue = chain.chainId ?? "";
    if (!chainValue) {
      continue;
    }
    if (sameChainId(chainValue, active)) {
      hasActiveOption = true;
    }
    const option = document.createElement("option");
    option.value = chainValue;
    option.textContent = networkOptionLabel(chain);
    networkSelect.append(option);
  }
  if (active && !hasActiveOption) {
    const option = document.createElement("option");
    option.value = active;
    option.textContent = networkOptionLabel(status.network ?? { chainId: active });
    networkSelect.prepend(option);
  }
  networkSelect.value = active;
  networkSelect.disabled = networkSelect.options.length === 0;
  updateNetworkSwitchState();
}

function updateNetworkSwitchState() {
  const target = networkSelect.value;
  switchNetworkButton.disabled =
    networkSelect.disabled || !target || sameChainId(target, latestStatus?.chainId);
}

function renderPortfolio(portfolio) {
  latestPortfolio = portfolio;
  const errors = portfolio.errors ?? [];
  const tokens = portfolio.tokens ?? [];
  const nativeBalance = portfolio.native?.balance;
  const tokenScan = portfolio.tokenScan ?? {};
  const hasRpc = Boolean(portfolio.rpc);
  const updatedAt = Date.now();

  if (portfolio.address) {
    accountAddress.textContent = portfolio.address;
  }
  if (portfolio.chainId) {
    chainId.textContent = portfolio.chainId;
  }
  if (nativeBalance) {
    accountBalance.textContent = formatNativeBalance(nativeBalance);
  } else if (!hasRpc) {
    accountBalance.textContent = "RPC missing";
  }

  portfolioSummary.textContent = portfolioSummaryText(portfolio);
  portfolioSummary.dataset.tone = errors.length > 0 ? (tokens.length > 0 ? "warn" : "bad") : "good";
  portfolioBlock.textContent = portfolio.blockNumber
    ? `Block ${formatHexInteger(portfolio.blockNumber)}`
    : hasRpc
      ? "Block unavailable"
      : "RPC missing";
  portfolioNative.textContent = nativeBalance ? formatNativeBalance(nativeBalance) : "-";
  const watchedCount = tokenScan.watched ?? tokens.filter((token) => token.watched).length;
  const walletStateLabel = walletStatePersistenceLabel(portfolio.walletState?.persistence);
  portfolioTokenCount.textContent = `${tokens.length} shown · ${tokenScan.nonzero ?? 0} nonzero · ${watchedCount} watched${walletStateLabel}`;
  portfolioUpdated.textContent = formatTime(updatedAt);

  portfolioAssets.replaceChildren();
  if (tokens.length === 0) {
    const empty = document.createElement("div");
    empty.className = "review-empty";
    empty.textContent = errors.length > 0 ? "No token balances available" : "No ERC-20 balances";
    portfolioAssets.append(empty);
  } else {
    for (const token of tokens) {
      portfolioAssets.append(renderPortfolioAsset(token));
    }
  }

  if (errors.length > 0) {
    const errorList = document.createElement("div");
    errorList.className = "portfolio-errors";
    for (const error of errors.slice(0, 3)) {
      const item = document.createElement("span");
      item.textContent = `${error.scope ?? "portfolio"}: ${error.message ?? "unavailable"}`;
      errorList.append(item);
    }
    portfolioAssets.append(errorList);
  }
  syncSelectedTokenFromPortfolio(tokens, portfolio.chainId);
  renderSessionReadiness();
}

function setPortfolioLoading() {
  portfolioSummary.textContent = "Loading";
  portfolioSummary.dataset.tone = "busy";
  portfolioBlock.textContent = "-";
  portfolioNative.textContent = "Loading...";
  portfolioTokenCount.textContent = "-";
  portfolioUpdated.textContent = "-";
}

function renderPortfolioError(error) {
  latestPortfolio = null;
  portfolioSummary.textContent = "Unavailable";
  portfolioSummary.dataset.tone = "bad";
  portfolioNative.textContent = "-";
  portfolioTokenCount.textContent = "-";
  portfolioUpdated.textContent = "-";
  portfolioAssets.replaceChildren();
  const empty = document.createElement("div");
  empty.className = "review-empty";
  empty.textContent = error?.message ?? error?.error?.message ?? "Portfolio unavailable";
  portfolioAssets.append(empty);
  renderSessionReadiness();
}

function renderPortfolioBaseline() {
  latestPortfolio = null;
  portfolioSummary.textContent = "Not loaded";
  portfolioSummary.dataset.tone = "idle";
  portfolioBlock.textContent = "-";
  portfolioNative.textContent = "-";
  portfolioTokenCount.textContent = "-";
  portfolioUpdated.textContent = "-";
  portfolioAssets.replaceChildren();
  const empty = document.createElement("div");
  empty.className = "review-empty";
  empty.textContent = "No portfolio snapshot";
  portfolioAssets.append(empty);
}

function setNativeSendState(label, tone, detail) {
  nativeSendStatus.textContent = label;
  nativeSendStatus.dataset.tone = tone;
  nativeSendDetail.dataset.tone = tone;
  nativeSendDetail.replaceChildren();
  const item = document.createElement("span");
  item.textContent = detail;
  nativeSendDetail.append(item);
}

function renderNativeSendResult(result) {
  if (result.status === "broadcast") {
    setNativeSendState(
      "Broadcast",
      "good",
      `${shortHash(result.transactionHash)} · ${result.amount} ${result.nativeSymbol ?? "ETH"}`,
    );
    return;
  }
  if (result.status === "failed") {
    setNativeSendState(
      "Failed",
      "bad",
      result.providerError?.message ?? "Native transfer failed after review",
    );
    return;
  }
  setNativeSendState("Done", "good", "Native transfer finished");
}

function renderNativeSendError(error) {
  setNativeSendState("Error", "bad", error?.message ?? error?.error?.message ?? String(error));
}

function canSendPortfolioToken(token) {
  return Boolean(
    token?.contractAddress &&
      Number.isInteger(token.metadata?.decimals) &&
      token.metadata.decimals >= 0 &&
      token.metadata.decimals <= 255,
  );
}

function tokenSymbol(token) {
  return token?.metadata?.symbol ?? "ERC-20";
}

function selectTokenForSend(token) {
  if (!canSendPortfolioToken(token)) {
    setTokenSendState("Unavailable", "bad", "Token decimals are required before transfer");
    return;
  }
  selectedTokenForSend = {
    contractAddress: token.contractAddress,
    symbol: tokenSymbol(token),
    name: token.metadata?.name ?? null,
    decimals: token.metadata.decimals,
    balance: token.balance ?? null,
    chainId: latestPortfolio?.chainId ?? latestStatus?.chainId ?? null,
    watched: Boolean(token.watched),
  };
  tokenSendAmount.placeholder = `0.0 ${selectedTokenForSend.symbol}`;
  renderTokenSendSelection();
  setTokenSendState("Ready", "good", `Review required before sending ${selectedTokenForSend.symbol}`);
}

function syncSelectedTokenFromPortfolio(tokens, chainId) {
  if (!selectedTokenForSend) {
    return;
  }
  if (chainId && selectedTokenForSend.chainId && !sameChainId(chainId, selectedTokenForSend.chainId)) {
    clearTokenSendSelection();
    return;
  }
  const fresh = tokens.find(
    (token) =>
      typeof token.contractAddress === "string" &&
      token.contractAddress.toLowerCase() === selectedTokenForSend.contractAddress.toLowerCase(),
  );
  if (!fresh || !canSendPortfolioToken(fresh)) {
    renderTokenSendSelection();
    return;
  }
  selectedTokenForSend = {
    ...selectedTokenForSend,
    symbol: tokenSymbol(fresh),
    name: fresh.metadata?.name ?? null,
    decimals: fresh.metadata.decimals,
    balance: fresh.balance ?? selectedTokenForSend.balance,
    watched: Boolean(fresh.watched),
    chainId: chainId ?? selectedTokenForSend.chainId,
  };
  renderTokenSendSelection();
}

function clearTokenSendSelection() {
  selectedTokenForSend = null;
  tokenSendSelected.replaceChildren();
  const title = document.createElement("strong");
  title.textContent = "Select from Portfolio";
  const detail = document.createElement("span");
  detail.textContent = "ERC-20 transfer";
  tokenSendSelected.append(title, detail);
  tokenSendAmount.placeholder = "0.0";
  tokenSendSubmitButton.disabled = true;
  setTokenSendState("No token", "idle", "Select an ERC-20 token from Portfolio");
}

function renderTokenSendSelection() {
  tokenSendSelected.replaceChildren();
  if (!selectedTokenForSend) {
    clearTokenSendSelection();
    return;
  }
  const title = document.createElement("strong");
  title.textContent = selectedTokenForSend.symbol;
  const contract = document.createElement("span");
  const balance = selectedTokenForSend.balance
    ? formatTokenBalance(
        selectedTokenForSend.balance,
        selectedTokenForSend.decimals,
        selectedTokenForSend.symbol,
      )
    : "balance unavailable";
  contract.textContent = `${shortAddress(selectedTokenForSend.contractAddress)} · ${balance}`;
  tokenSendSelected.append(title, contract);
  tokenSendSubmitButton.disabled = false;
}

function setTokenSendState(label, tone, detail) {
  tokenSendStatus.textContent = label;
  tokenSendStatus.dataset.tone = tone;
  tokenSendDetail.dataset.tone = tone;
  tokenSendDetail.replaceChildren();
  const item = document.createElement("span");
  item.textContent = detail;
  tokenSendDetail.append(item);
}

function renderTokenSendResult(result) {
  const symbol = result.symbol ?? selectedTokenForSend?.symbol ?? "token";
  if (result.status === "broadcast") {
    setTokenSendState("Broadcast", "good", `${shortHash(result.transactionHash)} · ${result.amount} ${symbol}`);
    return;
  }
  if (result.status === "failed") {
    setTokenSendState("Failed", "bad", result.providerError?.message ?? "Token transfer failed after review");
    return;
  }
  setTokenSendState("Done", "good", "Token transfer finished");
}

function renderTokenSendError(error) {
  setTokenSendState("Error", "bad", error?.message ?? error?.error?.message ?? String(error));
}

function renderTransactionActivity(activity) {
  const items = activity.items ?? [];
  latestTransactionActivity = items;
  latestActivityPersistence = activity.persistence ?? null;
  activityCount.textContent = `${items.length} transactions`;
  transactionActivity.replaceChildren();

  if (items.length === 0) {
    const empty = document.createElement("div");
    empty.className = "review-empty";
    empty.textContent = "No transaction activity";
    transactionActivity.append(empty);
  } else {
    for (const item of items) {
      transactionActivity.append(renderTransactionActivityItem(item));
    }
  }

  const refreshErrors = activity.receiptRefresh?.errors ?? [];
  if (refreshErrors.length > 0) {
    const errors = document.createElement("div");
    errors.className = "activity-errors";
    for (const error of refreshErrors.slice(0, 3)) {
      const row = document.createElement("span");
      const label = error.transactionHash ? shortHash(error.transactionHash) : error.scope ?? "receipt";
      row.textContent = `${label}: ${error.message ?? "unavailable"}`;
      errors.append(row);
    }
    transactionActivity.append(errors);
  }

  const smokeEnabled =
    latestStatus?.capabilities?.runtimeSmoke || latestStatus?.capabilities?.trustedAutosmoke;
  const latestActivityStatus = items[0]?.status;
  const activityReachedOutcome =
    latestActivityStatus && !["review_pending", "approved"].includes(latestActivityStatus);
  if (!transactionActivitySmokeReported && smokeEnabled && items.length > 0 && activityReachedOutcome) {
    transactionActivitySmokeReported = true;
    smokeReport("trusted_ui_activity_smoke", {
      count: items.length,
      latestStatus: latestActivityStatus,
      hasHash: Boolean(items[0]?.transactionHash),
      receiptStatus: items[0]?.receiptStatus ?? null,
    });
  }

  scheduleReceiptAutoRefresh(items);
  renderReceiptTrackingState(items);
  renderActivityPersistenceState(activity.persistence);
  renderProductOverview();
}

function renderTransactionActivityBaseline() {
  latestTransactionActivity = [];
  latestActivityPersistence = null;
  clearReceiptAutoRefreshTimer();
  activityCount.textContent = "0 transactions";
  renderReceiptTrackingState([]);
  renderActivityPersistenceState(null);
  transactionActivity.replaceChildren();
  const empty = document.createElement("div");
  empty.className = "review-empty";
  empty.textContent = "No transaction activity";
  transactionActivity.append(empty);
  renderProductOverview();
}

function scheduleReceiptAutoRefresh(items = latestTransactionActivity) {
  const refreshable = refreshableReceiptItems(items);
  if (refreshable.length === 0) {
    clearReceiptAutoRefreshTimer();
    return;
  }
  if (receiptAutoRefreshTimer || receiptAutoRefreshInFlight) {
    return;
  }

  const elapsedMs = lastReceiptRefreshAtUnixMs ? Date.now() - lastReceiptRefreshAtUnixMs : Infinity;
  const delayMs = Number.isFinite(elapsedMs)
    ? Math.max(TRANSACTION_RECEIPT_AUTO_REFRESH_MS - elapsedMs, TRANSACTION_RECEIPT_AUTO_REFRESH_MIN_DELAY_MS)
    : TRANSACTION_RECEIPT_AUTO_REFRESH_MIN_DELAY_MS;

  receiptAutoRefreshTimer = window.setTimeout(() => {
    receiptAutoRefreshTimer = null;
    refreshTransactionActivity(false, true).catch(() => {});
  }, delayMs);
}

function clearReceiptAutoRefreshTimer() {
  if (receiptAutoRefreshTimer) {
    window.clearTimeout(receiptAutoRefreshTimer);
    receiptAutoRefreshTimer = null;
  }
}

function refreshableReceiptItems(items = latestTransactionActivity) {
  return (items ?? []).filter((item) => {
    if (!item?.transactionHash) {
      return false;
    }
    const status = item.status ?? item.receiptStatus ?? "";
    return !TRANSACTION_ACTIVITY_FINAL_STATUSES.has(status);
  });
}

function renderReceiptTrackingState(items = latestTransactionActivity) {
  if (!activityReceiptTracking) {
    return;
  }
  const refreshable = refreshableReceiptItems(items);
  const latestReceipt = latestTransactionReceiptState(items);
  if (receiptAutoRefreshInFlight) {
    activityReceiptTracking.dataset.tone = "busy";
    activityReceiptTracking.textContent = "Checking receipts";
    return;
  }
  if (latestReceiptRefreshError && refreshable.length > 0) {
    activityReceiptTracking.dataset.tone = "warn";
    activityReceiptTracking.textContent = "Receipt check needs retry";
    return;
  }
  if (refreshable.length > 0) {
    activityReceiptTracking.dataset.tone = "warn";
    const checked = latestReceipt?.checkedAt ? ` · checked ${formatTime(latestReceipt.checkedAt)}` : "";
    activityReceiptTracking.textContent = `Auto-checking ${refreshable.length} pending receipt${
      refreshable.length === 1 ? "" : "s"
    }${checked}`;
    return;
  }
  if (latestReceipt?.status === "confirmed" || latestReceipt?.status === "included") {
    activityReceiptTracking.dataset.tone = "good";
    activityReceiptTracking.textContent = "Latest receipt confirmed";
    return;
  }
  if (latestReceipt?.status === "reverted") {
    activityReceiptTracking.dataset.tone = "bad";
    activityReceiptTracking.textContent = "Latest receipt reverted";
    return;
  }
  activityReceiptTracking.dataset.tone = "idle";
  activityReceiptTracking.textContent = "No pending receipts";
}

function latestTransactionReceiptState(items = latestTransactionActivity) {
  return (items ?? [])
    .filter((item) => item?.transactionHash)
    .map((item) => ({
      status: item.receipt?.status ?? item.receiptStatus ?? item.status,
      checkedAt: item.receiptCheckedAtUnixMs ?? item.updatedAtUnixMs ?? 0,
    }))
    .sort((left, right) => Number(right.checkedAt ?? 0) - Number(left.checkedAt ?? 0))[0];
}

function renderActivityPersistenceState(persistence) {
  if (!activityPersistence) {
    return;
  }
  if (persistence?.warning) {
    activityPersistence.dataset.tone = "warn";
    activityPersistence.textContent = "Activity save needs attention";
    return;
  }
  if (!persistence?.enabled) {
    activityPersistence.dataset.tone = "idle";
    activityPersistence.textContent = "Local session only";
    return;
  }
  if (persistence.restored && Number(persistence.itemsRestored) > 0) {
    activityPersistence.dataset.tone = "good";
    activityPersistence.textContent = `Restored ${persistence.itemsRestored} saved transaction${
      Number(persistence.itemsRestored) === 1 ? "" : "s"
    }`;
    return;
  }
  if (persistence.lastSavedAtUnixMs) {
    activityPersistence.dataset.tone = "good";
    activityPersistence.textContent = `Saved ${formatTime(persistence.lastSavedAtUnixMs)}`;
    return;
  }
  activityPersistence.dataset.tone = "good";
  activityPersistence.textContent = "Activity persistence ready";
}

function renderTransactionActivityItem(item) {
  const article = document.createElement("article");
  article.className = "activity-item";
  article.dataset.status = item.status ?? "review_pending";

  const header = document.createElement("div");
  header.className = "activity-item-header";
  const title = document.createElement("div");
  title.className = "activity-title";
  const name = document.createElement("strong");
  name.textContent = item.call ?? "eth_sendTransaction";
  const meta = document.createElement("span");
  meta.textContent = `${shortOrigin(item.origin) || "unknown origin"} · ${formatTime(
    item.updatedAtUnixMs,
  )}`;
  title.append(name, meta);
  const status = document.createElement("span");
  status.className = "activity-status";
  status.dataset.status = item.status ?? "review_pending";
  status.textContent = transactionActivityStatusLabel(item);
  header.append(title, status);

  const rows = [
    ["Hash", shortHash(item.transactionHash) || "-"],
    ["From", shortAddress(item.from ?? item.address)],
    ["To", shortAddress(item.to)],
    ["Value", item.value ? formatNativeBalance(item.value) : "-"],
    ["Policy", item.policyDecision],
    ["Simulation", item.simulationStatus],
    ["Receipt", transactionReceiptLabel(item)],
  ];
  article.append(header, summaryGrid(rows));

  const guidance = renderTransactionActivityGuidance(item.guidance);
  if (guidance) {
    article.append(guidance);
  }

  if (item.error) {
    const error = document.createElement("div");
    error.className = "activity-error";
    error.textContent = item.error;
    article.append(error);
  }

  return article;
}

function renderTransactionActivityGuidance(guidance) {
  if (!guidance || typeof guidance !== "object") {
    return null;
  }
  const section = document.createElement("div");
  section.className = "activity-guidance";
  section.dataset.tone = guidance.tone ?? "warn";

  const header = document.createElement("div");
  header.className = "activity-guidance-head";
  const title = document.createElement("strong");
  title.textContent = guidance.title ?? "Transaction guidance";
  const action = document.createElement("span");
  action.textContent = userActionLabel(guidance.primaryAction, "Review");
  header.append(title, action);

  const message = document.createElement("p");
  message.textContent = guidance.message ?? "-";
  const next = document.createElement("div");
  next.className = "activity-guidance-next";
  next.textContent = guidance.nextStep ?? "Review the activity details before retrying.";
  section.append(header, message, next);
  return section;
}

function transactionActivityStatusLabel(item) {
  const labels = {
    review_pending: "Review",
    approved: "Approved",
    rejected: "Rejected",
    expired: "Expired",
    failed: "Failed",
    broadcast: "Broadcast",
    confirmed: "Confirmed",
    reverted: "Reverted",
    included: "Included",
  };
  return labels[item.status] ?? labelize(item.status ?? "pending");
}

function transactionReceiptLabel(item) {
  if (item.receipt) {
    const block = item.receipt.blockNumber ? ` block ${formatHexInteger(item.receipt.blockNumber)}` : "";
    const gas = item.receipt.gasUsed ? ` gas ${formatHexInteger(item.receipt.gasUsed)}` : "";
    return `${transactionActivityStatusLabel({ status: item.receipt.status })}${block}${gas}`;
  }
  if (item.receiptStatus) {
    const checked = item.receiptCheckedAtUnixMs ? ` · checked ${formatTime(item.receiptCheckedAtUnixMs)}` : "";
    return `${labelize(item.receiptStatus)}${checked}`;
  }
  return item.transactionHash ? "Not checked" : "-";
}

function walletStatePersistenceLabel(persistence) {
  if (!persistence) {
    return "";
  }
  if (persistence.warning) {
    return " · watched save warning";
  }
  if (!persistence.enabled) {
    return "";
  }
  if (persistence.restored && Number(persistence.watchedAssetsRestored) > 0) {
    const restored = Number(persistence.watchedAssetsRestored);
    return ` · ${restored} watched restored`;
  }
  if (persistence.lastSavedAtUnixMs) {
    return " · watched saved";
  }
  return " · watched saved locally";
}

function renderPortfolioAsset(token) {
  const item = document.createElement("article");
  item.className = "portfolio-asset";

  const metadata = token.metadata ?? {};
  const symbol = metadata.symbol ?? "ERC-20";
  const title = document.createElement("div");
  title.className = "portfolio-asset-title";
  const name = document.createElement("strong");
  name.textContent = symbol;
  const contract = document.createElement("span");
  contract.textContent = `${shortAddress(token.contractAddress)}${token.watched ? " · watched" : ""}`;
  title.append(name, contract);

  const amount = document.createElement("div");
  amount.className = "portfolio-asset-amount";
  amount.textContent = formatTokenBalance(token.balance, metadata.decimals, symbol);

  const detail = document.createElement("div");
  detail.className = "portfolio-asset-detail";
  const watched = token.watched
    ? ` · watched${token.watchOrigin ? ` from ${shortOrigin(token.watchOrigin)}` : ""}`
    : "";
  detail.textContent = metadata.name
    ? `${metadata.name}${token.metadataError ? " · metadata partial" : ""}${watched}`
    : token.metadataError
      ? `Metadata unavailable · ${shortAddress(token.contractAddress)}${watched}`
      : `${shortAddress(token.contractAddress)}${watched}`;

  const actions = document.createElement("div");
  actions.className = "portfolio-asset-actions";
  const send = document.createElement("button");
  send.type = "button";
  send.className = "portfolio-asset-send";
  send.textContent = "Send";
  send.disabled = !canSendPortfolioToken(token);
  send.title = send.disabled ? "Token decimals required" : `Send ${symbol}`;
  send.addEventListener("click", () => {
    selectTokenForSend(token);
  });
  actions.append(send);

  item.append(title, amount, detail, actions);
  return item;
}

function portfolioSummaryText(portfolio) {
  if (!portfolio.rpc) {
    return "RPC missing";
  }
  const native = portfolio.native?.balance ? formatNativeBalance(portfolio.native.balance) : "ETH -";
  const tokenCount = portfolio.tokens?.length ?? 0;
  const errorCount = portfolio.errors?.length ?? 0;
  if (errorCount > 0) {
    return `${native} · ${tokenCount} tokens · ${errorCount} issue${errorCount === 1 ? "" : "s"}`;
  }
  return `${native} · ${tokenCount} tokens`;
}

function renderReviewQueue(queue) {
  const requests = queue.requests ?? [];
  latestReviewRequests = requests;
  reviewCount.textContent = `${requests.length} approval${requests.length === 1 ? "" : "s"}`;
  updateWorkspaceReviewCounts();
  updatePendingReviewSurface(requests);
  reviewList.replaceChildren();
  renderSessionReadiness();
  renderCompatibilityStatus();

  if (requests.length === 0) {
    const empty = document.createElement("div");
    empty.className = "review-empty";
    empty.textContent = "No wallet approvals waiting";
    reviewList.append(empty);
    return;
  }

  for (const request of requests) {
    reviewList.append(renderReviewRequest(request));
  }
  syncPendingReviewFocus(requests);
}

function updatePendingReviewSurface(requests) {
  const pending = requests.some((request) => request.status === "pending");
  document.body.dataset.pendingReview = pending ? "true" : "false";
  if (!pending) {
    lastPendingReviewKey = "";
  }
}

function syncPendingReviewFocus(requests) {
  const pending = requests.filter((request) => request.status === "pending");
  const pendingKey = pending.map((request) => request.id ?? request.providerRequestId ?? "").join("|");
  if (pendingKey === lastPendingReviewKey) {
    return;
  }
  lastPendingReviewKey = pendingKey;
  if (pending.length === 0 || activeWorkspace !== "defi") {
    return;
  }
  window.setTimeout(() => {
    focusReviewPanel();
  }, 0);
}

function focusReviewPanel() {
  const panel = document.querySelector(".review-panel");
  if (!panel || panel.hidden) {
    return;
  }
  panel.scrollIntoView({ behavior: "smooth", block: "start" });
  const firstAction = panel.querySelector(".review-item button:not([disabled])");
  if (firstAction instanceof HTMLElement) {
    window.setTimeout(() => {
      firstAction.focus({ preventScroll: true });
    }, 180);
  }
}

function renderConnectedSites(value) {
  const origins = value.origins ?? [];
  latestConnectedOrigins = origins;
  connectedSites.replaceChildren();
  renderSessionReadiness();
  renderCompatibilityStatus();
  if (origins.length === 0) {
    const empty = document.createElement("li");
    empty.className = "connected-site-empty";
    empty.textContent = "No sites connected";
    connectedSites.append(empty);
    return;
  }

  for (const origin of origins) {
    const item = document.createElement("li");
    const text = document.createElement("span");
    text.textContent = origin;
    const disconnect = document.createElement("button");
    disconnect.type = "button";
    disconnect.textContent = "Disconnect";
    disconnect.addEventListener("click", () => {
      revokeConnectedSite(origin).catch(() => {});
    });
    item.append(text, disconnect);
    connectedSites.append(item);
  }
}

function renderProviderEvents(value) {
  const events = value.events ?? [];
  latestProviderEvents = events;
  syncWalletConnectionErrorFromProviderEvents(events);
  providerEventCount.textContent = `${events.length} events`;
  providerEvents.replaceChildren();
  renderSessionReadiness();
  renderCompatibilityStatus();
  if (events.length === 0) {
    const empty = document.createElement("div");
    empty.className = "review-empty";
    empty.textContent = "No provider events";
    providerEvents.append(empty);
    return;
  }

  for (const event of [...events].reverse()) {
    providerEvents.append(renderProviderEvent(event));
  }
}

function syncWalletConnectionErrorFromProviderEvents(events) {
  if (latestAccount?.address || walletConnectionPending) {
    return;
  }
  const latestConnectEvent = lastItem(
    events.filter((event) => event.kind === "provider_request" && event.method === "framkey_getAccount"),
  );
  if (latestConnectEvent?.status === "error" && latestConnectEvent.errorMessage) {
    walletConnectionError = {
      code: latestConnectEvent.errorCode ?? 4900,
      message: latestConnectEvent.errorMessage,
    };
  }
}

function renderSessionReadiness() {
  if (!readinessGrid) {
    renderProductOverview();
    return;
  }

  const connectedOrigin = latestConnectedOrigins[0] ?? null;
  const providerInjected = latestProviderEvents.find((event) => event.kind === "provider_injected");
  const latestProviderRequest = lastItem(
    latestProviderEvents.filter((event) => event.kind === "provider_request"),
  );
  const latestSign = lastItem(
    latestReviewRequests.filter((request) =>
      ["personal_sign", "typed_data"].includes(request.kind),
    ),
  );
  const latestTx = lastItem(
    latestReviewRequests.filter((request) => request.kind === "transaction"),
  );
  const pending = latestReviewRequests.filter((request) => request.status === "pending");

  const walletReady = Boolean(latestStatus?.wallet);
  const accountReady = Boolean(latestAccount?.address || connectedOrigin);
  const rpcReady = latestRpcHealth ? Boolean(latestRpcHealth.healthy) : Boolean(latestStatus?.rpc);
  const rpcReadinessLabel = latestRpcHealth
    ? rpcHealthReadinessText(latestRpcHealth)
    : rpcReady
      ? "Alchemy"
      : "Missing";
  const providerReady = Boolean(providerInjected);
  const reviewReady = pending.length === 0;

  readinessGrid.replaceChildren(
    readinessItem("Wallet", walletReady ? "Ready" : "Load", walletReady ? "good" : "warn"),
    readinessItem("RPC", rpcReadinessLabel, rpcReady ? "good" : "bad"),
    readinessItem("dApp", providerReady ? "Injected" : "Waiting", providerReady ? "good" : "warn"),
    readinessItem("Review", reviewReady ? "Clear" : `${pending.length} pending`, reviewReady ? "good" : "warn"),
  );

  const sessionTone = sessionToneForState({
    walletReady,
    rpcReady,
    providerReady,
    pendingCount: pending.length,
  });
  sessionSummary.textContent = sessionSummaryText(sessionTone, pending.length);
  sessionSummary.dataset.tone = sessionTone;
  sessionDapp.textContent = latestProviderOrigin() ?? dappSessionOneLine(latestDappSession) ?? activeDappTarget;
  sessionAccountGrant.textContent = connectedOrigin
    ? `${shortOrigin(connectedOrigin)} connected`
    : accountReady
      ? "Trusted UI only"
      : "No connected site";
  sessionProvider.textContent = providerReady
    ? providerEventOneLine(latestProviderRequest ?? providerInjected)
    : "No provider injection yet";
  sessionSign.textContent = latestSign ? reviewOneLine(latestSign) : "No signature request";
  sessionTransaction.textContent = latestTx ? reviewOneLine(latestTx) : "No transaction request";
  sessionNextAction.textContent = nextSessionAction({
    walletReady,
    rpcReady,
    providerReady,
    accountReady,
    connectedOrigin,
    pending,
    latestTransaction: latestTx,
    latestActivity: latestTransactionActivity[0],
  });
  renderProductOverview();
}

function renderCompatibilityStatus() {
  if (!compatibilityGrid) {
    return;
  }
  const targetStates = COMPATIBILITY_TARGETS.map(compatibilityStateForTarget);
  compatibilityGrid.replaceChildren(...targetStates.map(renderCompatibilityTarget));

  const tested = targetStates.filter((state) => state.evidenceCount > 0);
  const ready = targetStates.filter((state) => state.tone === "good");
  if (tested.length === 0) {
    compatibilitySummary.textContent = "No runs";
    compatibilitySummary.dataset.tone = "busy";
    return;
  }
  compatibilitySummary.textContent = `${ready.length}/${COMPATIBILITY_TARGETS.length} ready`;
  compatibilitySummary.dataset.tone = ready.length === COMPATIBILITY_TARGETS.length ? "good" : "warn";
}

function compatibilityStateForTarget(target) {
  const events = latestProviderEvents.filter((event) => eventMatchesOrigin(event.origin, target.origin));
  const reviews = latestReviewRequests.filter((request) =>
    eventMatchesOrigin(request.origin, target.origin),
  );
  const provider = compatibilityStepFromEvent(
    "Provider",
    events.find((event) => event.kind === "provider_injected"),
  );
  const read = compatibilityStepFromEvent(
    "Read RPC",
    lastItem(
      events.filter(
        (event) =>
          event.kind === "provider_request" &&
          ["eth_chainId", "eth_blockNumber", "eth_getBalance", "eth_call"].includes(event.method),
      ),
    ),
  );
  const connect = compatibilityConnectStep(target, events, reviews);
  const watchAsset = compatibilityRequestStep("Token", events, reviews, "wallet_watchAsset");
  const sign = compatibilityRequestStep("Sign", events, reviews, "personal_sign");
  const permit = compatibilityRequestStep("Permit", events, reviews, "eth_signTypedData_v4");
  const transaction = compatibilityTransactionStep(events, reviews);
  const steps = [provider, read, connect, watchAsset, permit, sign, transaction];
  const checking = activeCompatibilityChecks.has(target.key);
  return {
    target,
    steps,
    evidenceCount: events.length + reviews.length,
    lastSeen: lastCompatibilityTime(events, reviews),
    checking,
    guidance: compatibilityGuidance({ checking, steps, evidenceCount: events.length + reviews.length }),
    tone: checking ? "busy" : compatibilityTone(steps, events.length + reviews.length),
  };
}

function renderCompatibilityTarget(state) {
  const item = document.createElement("article");
  item.className = "compatibility-target";
  item.dataset.tone = state.tone;

  const header = document.createElement("div");
  header.className = "compatibility-target-header";
  const title = document.createElement("div");
  const name = document.createElement("strong");
  const meta = document.createElement("span");
  name.textContent = state.target.label;
  meta.textContent = state.checking
    ? "Checking provider"
    : state.lastSeen
      ? `Last ${formatTime(state.lastSeen)}`
      : "No run evidence";
  title.append(name, meta);

  const actions = document.createElement("div");
  actions.className = "compatibility-target-actions";
  const open = document.createElement("button");
  open.type = "button";
  open.textContent = "Open";
  open.addEventListener("click", () => {
    if (state.target.key === "uniswap") {
      dappUrl.value = "https://app.uniswap.org/";
    }
    if (state.target.key === "aave") {
      dappUrl.value = "https://app.aave.com/";
    }
    openDapp(state.target.openUrl, state.target.label).catch(() => {});
  });
  const check = document.createElement("button");
  check.type = "button";
  check.textContent = state.checking ? "Checking" : "Check";
  check.disabled = state.checking;
  check.title = "Run a read-only provider/RPC compatibility check";
  check.addEventListener("click", () => {
    runCompatibilityCheck(state.target).catch(() => {});
  });
  actions.append(open, check);
  header.append(title, actions);

  const steps = document.createElement("div");
  steps.className = "compatibility-steps";
  for (const step of state.steps) {
    steps.append(renderCompatibilityStep(step));
  }

  const guidance = renderCompatibilityGuidance(state.guidance);
  item.append(header);
  if (guidance) {
    item.append(guidance);
  }
  item.append(steps);
  return item;
}

function renderCompatibilityGuidance(guidance) {
  if (!guidance) {
    return null;
  }
  const section = document.createElement("div");
  section.className = "compatibility-guidance";
  section.dataset.tone = guidance.tone;

  const head = document.createElement("div");
  head.className = "compatibility-guidance-head";
  const title = document.createElement("strong");
  title.textContent = guidance.title;
  const action = document.createElement("span");
  action.textContent = guidance.action;
  head.append(title, action);

  const message = document.createElement("p");
  message.textContent = guidance.message;
  section.append(head, message);
  return section;
}

function renderCompatibilityStep(step) {
  const item = document.createElement("div");
  item.className = "compatibility-step";
  item.dataset.tone = step.tone;
  const label = document.createElement("span");
  const value = document.createElement("strong");
  label.textContent = step.label;
  value.textContent = step.text;
  item.append(label, value);
  if (step.detail) {
    item.title = step.detail;
  }
  return item;
}

function compatibilityGuidance({ checking, steps, evidenceCount }) {
  const provider = compatibilityStep(steps, "Provider");
  const read = compatibilityStep(steps, "Read RPC");
  const connect = compatibilityStep(steps, "Connect");
  const permit = compatibilityStep(steps, "Permit");
  const sign = compatibilityStep(steps, "Sign");
  const tx = compatibilityStep(steps, "Tx");

  if (checking) {
    return {
      tone: "busy",
      title: "Checking provider",
      action: "Wait",
      message: "FRAMKey is opening the dApp and running a read-only provider/RPC probe.",
    };
  }
  if (evidenceCount === 0) {
    return {
      tone: "idle",
      title: "Not checked",
      action: "Check",
      message: "Run a read-only check to confirm provider injection and RPC access.",
    };
  }
  if (provider?.tone === "bad" || read?.tone === "bad") {
    return {
      tone: "bad",
      title: provider?.tone === "bad" ? "Provider issue" : "Read RPC issue",
      action: provider?.tone === "bad" ? "Open and Check" : "Check RPC",
      message:
        provider?.tone === "bad"
          ? "The page has not shown a working FRAMKey provider yet."
          : "The dApp reached FRAMKey but a read-only RPC request failed.",
    };
  }
  if (provider?.tone !== "good" || read?.tone !== "good") {
    return {
      tone: "warn",
      title: "Evidence incomplete",
      action: "Check",
      message: "Run the read-only check again after the dApp finishes loading.",
    };
  }
  if (tx?.tone === "good") {
    return {
      tone: "good",
      title: "Transaction path proven",
      action: "Use dApp",
      message: "Provider, read RPC, account access, signing, and transaction broadcast evidence are present.",
    };
  }
  if (tx?.tone === "warn") {
    return {
      tone: "good",
      title: "Wallet flow reached transaction",
      action: "Fund or Retry",
      message: "The transaction reached FRAMKey signing/broadcast; the latest warning is expected for an unfunded mock account.",
    };
  }
  if (connect?.tone === "good" && (permit?.tone === "good" || sign?.tone === "good")) {
    return {
      tone: "good",
      title: "Signing path proven",
      action: "Use dApp",
      message: "The dApp can connect to FRAMKey and at least one controlled signing path has passed.",
    };
  }
  if (connect?.tone === "good") {
    return {
      tone: "warn",
      title: "Connected, signing untested",
      action: "Use dApp",
      message: "Account access is granted. Trigger a Permit, message signature, or transaction to complete the check.",
    };
  }
  return {
    tone: "warn",
    title: "Read-ready",
    action: "Connect in dApp",
    message: "Provider injection and read RPC are working. Connect the dApp when you are ready to test approvals and signing.",
  };
}

function compatibilityStep(steps, label) {
  return steps.find((step) => step.label === label) ?? null;
}

function compatibilityStepFromEvent(label, event) {
  if (!event) {
    return { label, text: "Not seen", tone: "idle" };
  }
  if (event.status === "ok" || event.status === "recorded") {
    return {
      label,
      text: event.status === "ok" ? "Passed" : "Seen",
      tone: "good",
      detail: providerEventOneLine(event),
    };
  }
  return {
    label,
    text: "Error",
    tone: "bad",
    detail: providerEventOneLine(event),
  };
}

function compatibilityConnectStep(target, events, reviews) {
  if (latestConnectedOrigins.some((origin) => eventMatchesOrigin(origin, target.origin))) {
    return { label: "Connect", text: "Granted", tone: "good" };
  }
  const connectEvent = lastItem(
    events.filter(
      (event) => event.kind === "provider_request" && event.method === "eth_requestAccounts",
    ),
  );
  if (connectEvent) {
    return compatibilityStepFromEvent("Connect", connectEvent);
  }
  const connectReview = lastItem(reviews.filter((request) => request.kind === "account_connection"));
  if (connectReview) {
    return compatibilityStepFromReview("Connect", connectReview);
  }
  return { label: "Connect", text: "Not run", tone: "idle" };
}

function compatibilityRequestStep(label, events, reviews, method) {
  const event = lastItem(
    events.filter((item) => item.kind === "provider_request" && item.method === method),
  );
  if (event) {
    return compatibilityStepFromEvent(label, event);
  }
  const review = lastItem(reviews.filter((request) => request.method === method));
  if (review) {
    return compatibilityStepFromReview(label, review);
  }
  return { label, text: "Not run", tone: "idle" };
}

function compatibilityTransactionStep(events, reviews) {
  const event = lastItem(
    events.filter(
      (item) => item.kind === "provider_request" && item.method === "eth_sendTransaction",
    ),
  );
  const review = lastItem(reviews.filter((request) => request.kind === "transaction"));
  if (event?.status === "ok") {
    return {
      label: "Tx",
      text: "Broadcast",
      tone: "good",
      detail: providerEventOneLine(event),
    };
  }
  if (event?.status === "error" && event.errorCode === -32003) {
    return {
      label: "Tx",
      text: "Broadcast error",
      tone: "warn",
      detail: providerEventOneLine(event),
    };
  }
  if (review) {
    return compatibilityStepFromReview("Tx", review);
  }
  if (event) {
    return compatibilityStepFromEvent("Tx", event);
  }
  return { label: "Tx", text: "Not run", tone: "idle" };
}

function compatibilityStepFromReview(label, review) {
  const status = String(review.status ?? "pending").replaceAll("_", " ");
  if (review.status === "signed" || review.status === "completed") {
    return { label, text: status, tone: "good", detail: formatReviewReason(review) };
  }
  if (review.status === "approved" || review.status === "pending") {
    return { label, text: status, tone: "warn", detail: formatReviewReason(review) };
  }
  if (review.status === "sign_failed" && review.kind === "transaction") {
    return { label, text: "Broadcast error", tone: "warn", detail: formatReviewReason(review) };
  }
  return { label, text: status, tone: "bad", detail: formatReviewReason(review) };
}

function compatibilityTone(steps, evidenceCount) {
  if (evidenceCount === 0) {
    return "idle";
  }
  const requiredSteps = steps.filter((step) => !(step.label === "Token" && step.tone === "idle"));
  if (requiredSteps.some((step) => step.tone === "bad")) {
    return "bad";
  }
  if (requiredSteps.some((step) => step.tone === "warn")) {
    return "warn";
  }
  if (requiredSteps.length > 0 && requiredSteps.every((step) => step.tone === "good")) {
    return "good";
  }
  return "busy";
}

function lastCompatibilityTime(events, reviews) {
  const eventTime = lastItem(events)?.unixMs ?? null;
  const reviewTime = lastItem(reviews)?.createdAtUnixMs ?? null;
  if (eventTime == null) {
    return reviewTime;
  }
  if (reviewTime == null) {
    return eventTime;
  }
  return Math.max(eventTime, reviewTime);
}

function eventMatchesOrigin(value, origin) {
  if (typeof value !== "string") {
    return false;
  }
  if (value === origin) {
    return true;
  }
  return origin === "tauri://localhost" && value.startsWith("tauri://localhost");
}

function readinessItem(label, value, tone) {
  const item = document.createElement("article");
  item.className = "readiness-item";
  item.dataset.tone = tone;
  const title = document.createElement("span");
  const body = document.createElement("strong");
  title.textContent = label;
  body.textContent = value;
  item.append(title, body);
  return item;
}

function sessionToneForState({ walletReady, rpcReady, providerReady, pendingCount }) {
  if (pendingCount > 0) {
    return "warn";
  }
  if (walletReady && rpcReady && providerReady) {
    return "good";
  }
  if (!rpcReady) {
    return "bad";
  }
  return "busy";
}

function sessionSummaryText(tone, pendingCount) {
  if (pendingCount > 0) {
    return `${pendingCount} pending`;
  }
  if (tone === "good") {
    return "Ready";
  }
  if (tone === "bad") {
    return "Needs RPC";
  }
  return "Waiting";
}

function latestProviderOrigin() {
  return lastItem(latestProviderEvents.filter((event) => event.origin && event.origin !== "null"))
    ?.origin;
}

function providerEventOneLine(event) {
  if (!event) {
    return "No provider event";
  }
  const method = event.method ?? labelize(event.kind ?? "event");
  const status = event.status ?? "recorded";
  const duration = event.durationMs != null ? ` ${event.durationMs}ms` : "";
  const error = event.errorMessage ? ` · ${event.errorMessage}` : "";
  return `${method} ${status}${duration}${error}`;
}

function reviewOneLine(request) {
  const status = String(request.status ?? "pending").replaceAll("_", " ");
  const origin = request.origin ? ` · ${shortOrigin(request.origin)}` : "";
  if (request.kind === "transaction") {
    const policy = request.summary?.policy?.decision
      ? ` · ${String(request.summary.policy.decision).replaceAll("_", " ")}`
      : "";
    return `${status}${policy}${origin}`;
  }
  return `${status}${origin}`;
}

function nextSessionAction({
  walletReady,
  rpcReady,
  providerReady,
  accountReady,
  connectedOrigin,
  pending,
  latestTransaction,
  latestActivity,
}) {
  const pendingTransaction =
    pending.find((request) => request.kind === "transaction") ??
    (latestTransaction?.status === "pending" ? latestTransaction : null);
  const pendingGuidance = pendingTransaction?.summary?.guidance;
  if (pendingGuidance?.blocked || pendingGuidance?.requiresHighRisk) {
    return pendingGuidance.nextStep ?? pendingGuidance.message ?? "Review pending transaction";
  }
  if (pending.length > 0) {
    return "Review pending request";
  }
  const activityAction = nextSessionActionFromActivity(latestActivity);
  if (activityAction) {
    return activityAction;
  }
  if (!walletReady) {
    return "Unlock wallet";
  }
  if (!rpcReady) {
    return "Check network";
  }
  if (!providerReady) {
    return "Choose an app";
  }
  if (!connectedOrigin) {
    return accountReady ? "Connect in app" : "Unlock wallet";
  }
  return "Use app";
}

function nextSessionActionFromActivity(activity) {
  if (!activity?.guidance) {
    return null;
  }
  const status = activity.status ?? activity.guidance.status;
  if (["failed", "reverted", "expired", "rejected", "broadcast"].includes(status)) {
    return activity.guidance.nextStep ?? activity.guidance.message ?? null;
  }
  return null;
}

function dappTargetLabel(url) {
  if (!url || url === "local" || url === "framkey://local-dapp") {
    return "Local Test";
  }
  if (url === "uniswap" || String(url).includes("app.uniswap.org")) {
    return "Uniswap";
  }
  if (url === "aave" || String(url).includes("app.aave.com")) {
    return "Aave";
  }
  return url;
}

function lastItem(items) {
  return items.length > 0 ? items[items.length - 1] : null;
}

function renderProviderEvent(event) {
  const item = document.createElement("article");
  item.className = "provider-event";
  item.dataset.status = event.status ?? "recorded";

  const header = document.createElement("div");
  header.className = "provider-event-header";
  const title = document.createElement("strong");
  title.textContent = event.method ?? labelize(event.kind ?? "event");
  const status = document.createElement("span");
  status.textContent = event.status ?? "recorded";
  header.append(title, status);

  const rows = [
    ["Kind", event.kind],
    ["Origin", event.origin],
    ["URL", event.url],
    ["Result", providerEventResult(event)],
    ["Error", providerEventError(event)],
    ["Duration", event.durationMs != null ? `${event.durationMs}ms` : null],
    ["Time", event.unixMs ? new Date(event.unixMs).toLocaleTimeString() : null],
  ];
  item.append(header, summaryGrid(rows));

  if (event.detail && typeof event.detail === "object") {
    const details = document.createElement("details");
    details.className = "review-params";
    const summary = document.createElement("summary");
    summary.textContent = "Telemetry detail";
    const body = document.createElement("pre");
    body.textContent = JSON.stringify(event.detail, null, 2);
    details.append(summary, body);
    item.append(details);
  }

  return item;
}

function providerEventResult(event) {
  if (!event.resultKind) {
    return null;
  }
  return event.resultPreview ? `${event.resultKind}: ${event.resultPreview}` : event.resultKind;
}

function providerEventError(event) {
  if (event.errorCode == null && !event.errorMessage) {
    return null;
  }
  return `${valueOrDash(event.errorCode)} ${valueOrDash(event.errorMessage)}`.trim();
}

function renderRecoveryOutcome(result, options = {}) {
  if (options.remember !== false) {
    rememberRecoveryOutcome(result);
  }
  renderRecoveryPanel();
}

function renderRecoveryState(result) {
  latestRecoveryBackupOutcome = result.backupOutcome ?? null;
  latestRecoveryDrillOutcome = result.drillOutcome ?? null;
  latestRecoveryRecoverOutcome = result.recoverOutcome ?? null;
  latestRecoveryPersistence = result.persistence ?? null;
  renderRecoveryPanel();
}

async function reportRecoveryStateRestoreSmoke(result) {
  if (recoveryStateRestoreSmokeReported || !result.persistence?.restored) {
    return;
  }
  recoveryStateRestoreSmokeReported = true;
  await smokeReport("trusted_ui_recovery_state_restored", {
    backupSet: result.backupOutcome?.recoveryBackups?.backupSetId,
    shareFileCount: result.backupOutcome?.recoveryBackups?.shareFileCount,
    drill: recoveryDrillStatus(result.drillOutcome),
    recover: Boolean(result.recoverOutcome),
    warning: result.persistence?.warning,
  });
}

function rememberRecoveryOutcome(result) {
  latestRecoveryPersistence = markRecoveryPersistenceCurrent(latestRecoveryPersistence);
  if (result.operation === "create_keychain_vault") {
    latestRecoveryBackupOutcome = result;
    latestRecoveryDrillOutcome = null;
    latestRecoveryRecoverOutcome = null;
    return;
  }
  if (result.operation === "recovery_smoke_pack") {
    latestRecoveryBackupOutcome = result;
    latestRecoveryDrillOutcome = result.recommendedDrill ?? null;
    latestRecoveryRecoverOutcome = null;
    return;
  }
  if (result.operation === "validate_recovery_set") {
    latestRecoveryDrillOutcome = result;
    return;
  }
  if (result.operation === "recover_keychain_vault") {
    latestRecoveryRecoverOutcome = result;
  }
}

function markRecoveryPersistenceCurrent(persistence) {
  if (!persistence) {
    return null;
  }
  return {
    ...persistence,
    restored: false,
    lastSavedAtUnixMs: Date.now(),
  };
}

function renderRecoveryPanel() {
  recoveryPlan.replaceChildren();

  if (
    !latestRecoveryBackupOutcome &&
    !latestRecoveryDrillOutcome &&
    !latestRecoveryRecoverOutcome
  ) {
    if (recoveryPanel) {
      recoveryPanel.dataset.empty = "true";
    }
    renderRecoveryPolicyBaseline({ clearState: false });
    renderProductOverview();
    return;
  }

  if (recoveryPanel) {
    recoveryPanel.dataset.empty = "false";
  }
  recoveryPlan.append(recoveryHealthCard());

  if (latestRecoveryBackupOutcome) {
    renderRecoveryBackupPlan(latestRecoveryBackupOutcome);
  } else {
    renderRecoveryPolicyGuide();
  }
  if (latestRecoveryDrillOutcome) {
    renderRecoveryDrillResult(latestRecoveryDrillOutcome);
  }
  if (latestRecoveryRecoverOutcome) {
    renderRecoveryRewrapResult(latestRecoveryRecoverOutcome);
  }
  renderProductOverview();
}

function renderRecoveryBackupPlan(result) {
  const backups = result.recoveryBackups ?? {};
  const files = backups.files ?? [];
  const shares = files.filter((file) => file.kind === "bundle");
  const title =
    result.operation === "recovery_smoke_pack" ? "Recovery Smoke Pack" : "Vault Created";

  recoveryPlan.append(
    recoveryHeader(title, [
      ["Mode", result.developmentOnly ? "development only" : "keychain vault"],
      ["Backup set", backups.backupSetId],
      ["Wallet", backups.walletId],
      ["Generation", backups.generation],
      ["Backup files", backups.backupFileCount ?? backups.bundleFileCount ?? shares.length],
      ["Vault data", "embedded in each file"],
      ["Cloud alone", backups.cloudAloneRecovers === false ? "insufficient" : "unknown"],
      ["Cloud-only drill", recoveryDrillStatus(result.cloudOnlyDrill)],
      ["Recommended drill", recoveryDrillStatus(result.recommendedDrill)],
      ["Save hash", result.saveImageBlake3],
    ]),
  );
  recoveryPlan.append(recoveryReadinessCard(shares));
  recoveryPlan.append(recoveryPlacementChecklist(files));
  recoveryPlan.append(recoveryBackupDetailList(files));
}

function renderRecoveryPolicyBaseline(options = {}) {
  if (options.clearState !== false) {
    latestRecoveryBackupOutcome = null;
    latestRecoveryDrillOutcome = null;
    latestRecoveryRecoverOutcome = null;
  }
  recoveryPlan.replaceChildren(
    recoveryPolicyGuide(),
    recoveryReadinessCard([]),
    recoveryPlacementMatrix([]),
  );
}

function renderRecoveryPolicyGuide() {
  recoveryPlan.append(recoveryPolicyGuide(), recoveryReadinessCard([]), recoveryPlacementMatrix([]));
}

function recoveryPolicyGuide() {
  return recoveryHeader("Backup Policy", [
    ["Cloud", "iCloud + Google Drive"],
    ["Local", "1 local physical share"],
    ["Remote", "1 remote physical share"],
    ["Cloud alone", "insufficient"],
  ]);
}

function recoveryHealthCard() {
  const shares = currentRecoveryShares();
  const placement = recoveryReadinessState(shares);
  const recover = latestRecoveryRecoverOutcome;
  const health = recoveryHealthState(placement, recover);
  const section = document.createElement("section");
  section.className = "recovery-readiness-card recovery-health-card";
  section.dataset.tone = health.tone;

  const header = document.createElement("div");
  header.className = "recovery-readiness-header";
  const title = document.createElement("strong");
  title.textContent = health.title;
  const badge = document.createElement("span");
  badge.textContent = health.badge;
  header.append(title, badge);

  section.append(
    header,
    summaryGrid([
      ["Backup pack", latestRecoveryBackupOutcome ? "created" : "not created"],
      ["Plan state", recoveryPlanPersistenceLabel()],
      ["Placement", placement.badge],
      ["Restore path", placement.tone === "good" ? "ready" : placement.badge],
      ["Mac + GBA restore", recover ? "completed" : "not run"],
      ["Next", health.nextAction],
    ]),
  );

  return section;
}

function recoveryPlanPersistenceLabel() {
  if (latestRecoveryPersistence?.warning) {
    return "local restore warning";
  }
  if (latestRecoveryPersistence?.restored) {
    return "restored from this Mac";
  }
  if (latestRecoveryPersistence?.enabled && latestRecoveryBackupOutcome) {
    return "saved on this Mac";
  }
  if (latestRecoveryPersistence?.enabled) {
    return "ready to save";
  }
  return "not persisted";
}

function recoveryHealthState(placement, recover) {
  if (recover) {
    return {
      title: "Recovery Health",
      badge: "Recovered",
      tone: "good",
      nextAction: "Refresh account or connect dApp",
    };
  }
  if (!latestRecoveryBackupOutcome) {
    return {
      title: "Recovery Health",
      badge: "No pack",
      tone: "idle",
      nextAction: "Create vault + recovery",
    };
  }
  if (placement.tone === "good") {
    return {
      title: "Recovery Health",
      badge: "Ready",
      tone: "good",
      nextAction: "Ready for restore if this Mac or GBA needs recovery",
    };
  }
  return {
    title: "Recovery Health",
    badge: placement.badge,
    tone: placement.tone,
    nextAction: placement.nextAction,
  };
}

function recoveryDrillStatus(drill) {
  if (!drill) {
    return "not run";
  }
  return drill.canRecover ? "passed" : "failed";
}

function currentRecoveryShares() {
  return (
    latestRecoveryBackupOutcome?.recoveryBackups?.files?.filter(
      (item) => item.kind === "bundle",
    ) ?? []
  );
}

function recoveryPlacementMatrix(shares) {
  const section = document.createElement("section");
  section.className = "recovery-placement-grid";
  const groups = [
    {
      key: "cloud",
      label: "Cloud",
      rule: "2-of-2",
      tone: "cloud",
      count: shares.filter((share) => share.group === "cloud").length,
      placed: checkedRecoveryShares(shares, "cloud").length,
    },
    {
      key: "local_physical",
      label: "Local Physical",
      rule: "1 file",
      tone: "local",
      count: shares.filter((share) => share.group === "local_physical").length,
      placed: checkedRecoveryShares(shares, "local_physical").length,
    },
    {
      key: "remote_physical",
      label: "Remote Physical",
      rule: "1 file",
      tone: "remote",
      count: shares.filter((share) => share.group === "remote_physical").length,
      placed: checkedRecoveryShares(shares, "remote_physical").length,
    },
  ];
  for (const group of groups) {
    const item = document.createElement("article");
    item.className = "recovery-placement-item";
    item.dataset.tone = group.tone;
    const label = document.createElement("span");
    label.textContent = group.label;
    const value = document.createElement("strong");
    value.textContent =
      group.count > 0 ? `${group.placed}/${group.count} placed · ${group.rule}` : group.rule;
    item.append(label, value);
    section.append(item);
  }
  return section;
}

function recoveryPlacementChecklist(artifacts) {
  const shares = artifacts.filter((file) => file.kind === "bundle");
  const section = document.createElement("section");
  section.className = "recovery-placement-checklist";

  const header = document.createElement("div");
  header.className = "recovery-placement-checklist-header";
  const copy = document.createElement("div");
  const eyebrow = document.createElement("span");
  eyebrow.textContent = "Placement";
  const title = document.createElement("strong");
  title.textContent = "Move each backup to its real destination";
  copy.append(eyebrow, title);
  const state = recoveryReadinessState(shares);
  const badge = document.createElement("span");
  badge.dataset.tone = state.tone;
  badge.textContent = state.badge;
  header.append(copy, badge);

  const tasks = document.createElement("div");
  tasks.className = "recovery-placement-tasks";
  for (const task of recoveryPlacementTasks(artifacts)) {
    tasks.append(recoveryPlacementTaskCard(task));
  }

  section.append(header, tasks);
  return section;
}

function recoveryPlacementTasks(artifacts) {
  const shares = artifacts.filter((file) => file.kind === "bundle");
  const byRole = (role) => artifacts.filter((file) => recoveryArtifactRole(file) === role);
  return [
    {
      key: "icloud",
      label: "iCloud Drive",
      role: "Cloud 1",
      requirement: "1 file",
      tone: "cloud",
      mode: "all",
      files: byRole("icloud"),
      guidance: "Place backup-01.dat in iCloud Drive.",
    },
    {
      key: "google",
      label: "Google Drive",
      role: "Cloud 2",
      requirement: "1 file",
      tone: "cloud",
      mode: "all",
      files: byRole("google"),
      guidance: "Place backup-02.dat in Google Drive.",
    },
    {
      key: "local",
      label: "Local Physical",
      role: "Physical copy 1",
      requirement: "Choose 1",
      tone: "local",
      mode: "any",
      files: shares.filter((share) => share.group === "local_physical"),
      guidance: "Use a TF card or USB drive you control.",
    },
    {
      key: "remote",
      label: "Remote Physical",
      role: "Physical copy 2",
      requirement: "Choose 1",
      tone: "remote",
      mode: "any",
      files: shares.filter((share) => share.group === "remote_physical"),
      guidance: "Store away from the main vault, Mac, and cloud accounts.",
    },
  ];
}

function recoveryPlacementTaskCard(task) {
  const status = recoveryPlacementTaskState(task);
  const card = document.createElement("article");
  card.className = "recovery-placement-task";
  card.dataset.tone = status.tone;

  const header = document.createElement("div");
  header.className = "recovery-placement-task-header";
  const titleGroup = document.createElement("div");
  const role = document.createElement("span");
  role.textContent = task.role;
  const title = document.createElement("strong");
  title.textContent = task.label;
  const note = document.createElement("p");
  note.textContent = `${task.requirement} · ${task.guidance}`;
  titleGroup.append(role, title, note);
  const badge = document.createElement("span");
  badge.dataset.tone = status.tone;
  badge.textContent = status.label;
  header.append(titleGroup, badge);

  const files = document.createElement("div");
  files.className = "recovery-placement-task-files";
  if (task.files.length === 0) {
    const empty = document.createElement("div");
    empty.className = "recovery-placement-empty";
    empty.textContent = "No generated file for this destination";
    files.append(empty);
  } else {
    for (const file of task.files) {
      files.append(recoveryPlacementFileRow(file));
    }
  }

  card.append(header, files);
  return card;
}

function recoveryPlacementTaskState(task) {
  const placed = task.files.filter(isBackupPlaced).length;
  const required =
    task.mode === "all"
      ? task.files.length
      : task.mode === "single"
        ? Math.min(task.files.length, 1)
        : 1;
  if (task.files.length === 0) {
    return { label: "Missing", tone: "bad" };
  }
  if (placed >= required) {
    return { label: task.mode === "any" ? `${placed} selected` : "Stored", tone: "good" };
  }
  return {
    label:
      task.mode === "all"
        ? `${placed}/${task.files.length} stored`
        : task.mode === "any"
          ? `0/${task.files.length} selected`
          : "Needed",
    tone: "warn",
  };
}

function recoveryPlacementFileRow(file) {
  const row = document.createElement("div");
  row.className = "recovery-placement-file-row";

  const check = document.createElement("label");
  check.className = "backup-check-row recovery-placement-check";
  const input = document.createElement("input");
  input.type = "checkbox";
  input.checked = isBackupPlaced(file);
  input.addEventListener("change", () => {
    setBackupPlaced(file, input.checked);
    refreshRecoveryPlacementUi();
  });
  const text = document.createElement("span");
  text.textContent = isBackupPlaced(file) ? "Stored" : "Mark stored";
  check.append(input, text);

  const details = document.createElement("div");
  details.className = "recovery-placement-file-detail";
  const name = document.createElement("strong");
  name.textContent = fileNameFromPath(file.path) || file.member || "Recovery file";
  const meta = document.createElement("span");
  meta.textContent = `${recoveryArtifactKindLabel(file)} · ${file.member ?? "backup"} · ${shortHash(file.blake3)}`;
  const path = document.createElement("small");
  path.textContent = file.path ?? "-";
  details.append(name, meta, path);

  const reveal = document.createElement("button");
  reveal.type = "button";
  reveal.textContent = "Show";
  reveal.addEventListener("click", () => {
    revealPath(file.path).catch(() => {});
  });

  row.append(check, details, reveal);
  return row;
}

function recoveryBackupDetailList(files) {
  const details = document.createElement("details");
  details.className = "recovery-detail-panel";

  const summary = document.createElement("summary");
  const title = document.createElement("strong");
  title.textContent = "All generated files";
  const count = document.createElement("span");
  count.textContent = `${files.length} files`;
  summary.append(title, count);

  const list = document.createElement("ul");
  list.className = "recovery-file-list recovery-detail-list";
  for (const file of files) {
    const item = document.createElement("li");
    const detail = document.createElement("div");
    const name = document.createElement("strong");
    name.textContent = fileNameFromPath(file.path) || file.member || recoveryArtifactKindLabel(file);
    const meta = document.createElement("span");
    meta.textContent = [
      recoveryArtifactKindLabel(file),
      file.destination,
      shortHash(file.blake3),
    ]
      .filter(Boolean)
      .join(" · ");
    detail.append(name, meta);

    const reveal = document.createElement("button");
    reveal.type = "button";
    reveal.textContent = "Show";
    reveal.addEventListener("click", () => {
      revealPath(file.path).catch(() => {});
    });

    item.append(detail, reveal);
    list.append(item);
  }

  details.append(summary, list);
  return details;
}

function recoveryReadinessCard(shares) {
  const state = recoveryReadinessState(shares);
  const section = document.createElement("section");
  section.className = "recovery-readiness-card";
  section.dataset.tone = state.tone;

  const header = document.createElement("div");
  header.className = "recovery-readiness-header";
  const title = document.createElement("strong");
  title.textContent = state.title;
  const badge = document.createElement("span");
  badge.textContent = state.badge;
  header.append(title, badge);

  section.append(
    header,
    summaryGrid([
      ["iCloud", state.icloud ? "placed" : "needed"],
      ["Google", state.google ? "placed" : "needed"],
      ["Local", `${state.localCount}/${state.localTotal} placed`],
      ["Remote", `${state.remoteCount}/${state.remoteTotal} placed`],
      ["Cloud alone", state.cloudComplete && !state.hasPhysical ? "insufficient" : "not used alone"],
      ["Next", state.nextAction],
    ]),
  );

  const selected = recoverableRecoveryFileBucketsFromChecked(shares);
  if (recoveryFilesFromBuckets(selected).length > 0) {
    const useChecked = document.createElement("button");
    useChecked.type = "button";
    useChecked.textContent = "Use Checked Recovery Set";
    useChecked.addEventListener("click", () => {
      setRecoveryFileBuckets(selected);
      renderRecoveryInputStatus();
    });
    const actions = document.createElement("div");
    actions.className = "recovery-readiness-actions";
    actions.append(useChecked);
    section.append(actions);
  }

  return section;
}

function recoveryReadinessState(shares) {
  const placed = shares.filter(isBackupPlaced);
  const icloud = placed.some((share) => recoveryShareRole(share) === "icloud");
  const google = placed.some((share) => recoveryShareRole(share) === "google");
  const localCount = checkedRecoveryShares(shares, "local_physical").length;
  const remoteCount = checkedRecoveryShares(shares, "remote_physical").length;
  const localTotal = shares.filter((share) => share.group === "local_physical").length;
  const remoteTotal = shares.filter((share) => share.group === "remote_physical").length;
  const cloudComplete = icloud && google;
  const hasLocal = localCount > 0;
  const hasRemote = remoteCount > 0;
  const hasPhysical = hasLocal || hasRemote;
  const recoverable = (cloudComplete && hasPhysical) || (hasLocal && hasRemote);

  if (shares.length === 0) {
    return {
      title: "Recovery Not Created",
      badge: "Waiting",
      tone: "idle",
      icloud,
      google,
      localCount,
      localTotal,
      remoteCount,
      remoteTotal,
      cloudComplete,
      hasPhysical,
      nextAction: "Create vault + recovery",
    };
  }
  if (recoverable) {
    return {
      title: "Recovery Set Ready",
      badge: "Recoverable",
      tone: "good",
      icloud,
      google,
      localCount,
      localTotal,
      remoteCount,
      remoteTotal,
      cloudComplete,
      hasPhysical,
      nextAction: missingRecommendedBackup(shares) ?? "Ready for restore when needed",
    };
  }
  if (cloudComplete && !hasPhysical) {
    return {
      title: "Cloud Alone Is Not Enough",
      badge: "Needs physical",
      tone: "bad",
      icloud,
      google,
      localCount,
      localTotal,
      remoteCount,
      remoteTotal,
      cloudComplete,
      hasPhysical,
      nextAction: "Place one local or remote physical share",
    };
  }
  return {
    title: "Recovery Set Incomplete",
    badge: "Incomplete",
    tone: "warn",
    icloud,
    google,
    localCount,
    localTotal,
    remoteCount,
    remoteTotal,
    cloudComplete,
    hasPhysical,
    nextAction: "Check both cloud shares plus one physical, or local plus remote physical",
  };
}

function renderRecoveryRewrapResult(result) {
  recoveryPlan.append(
    recoveryHeader("Vault Recovered", [
      ["Vault backup", result.vaultBackupPath],
      ["Backup hash", result.vaultBackupBlake3],
      ["Shares used", result.recoveryShareFileCount],
      ["Wallet secret", result.walletSecretTouched === false ? "not touched" : "unknown"],
      [
        "Share bytes",
        result.recoveryShareBytesPrinted === false ? "not printed" : "check output",
      ],
      ["Save hash", result.saveImageBlake3],
      ["Keychain", result.keychain?.accessPolicy],
    ]),
  );

  const files = document.createElement("ul");
  files.className = "recovery-file-list";
  for (const path of result.recoveryFiles ?? []) {
    const item = document.createElement("li");
    const label = document.createElement("strong");
    const value = document.createElement("span");
    label.textContent = "Used";
    value.textContent = path;
    item.append(label, value);
    files.append(item);
  }
  recoveryPlan.append(files);
}

async function revealPath(path) {
  if (!path) {
    renderError(new Error("No path to reveal"));
    return;
  }
  await invokeCommand("framkey_reveal_path", {
    request: { path },
  });
}

function renderRecoveryDrillResult(result) {
  recoveryPlan.append(
    recoveryHeader("Recovery Set Check", [
      ["Status", result.canRecover ? "recoverable" : "not recoverable"],
      ["Shares checked", result.recoveryShareFileCount],
      ["Groups", (result.satisfiedGroups ?? []).join(", ") || "none"],
      ["Backup set", result.backupSetId],
      ["Wallet", result.walletId],
      ["Generation", result.generation],
      ["Vault device", result.configuredVaultDeviceTouched === false ? "not touched" : "unknown"],
      ["Wallet secret", result.walletSecretTouched === false ? "not touched" : "unknown"],
      ["RRK", result.recoveryRootKeyPrinted === false ? "not printed" : "check output"],
    ]),
  );

  const card = document.createElement("section");
  card.className = "recovery-readiness-card";
  card.dataset.tone = result.canRecover ? "good" : "bad";
  const header = document.createElement("div");
  header.className = "recovery-readiness-header";
  const title = document.createElement("strong");
  title.textContent = result.canRecover ? "Recovery Drill Passed" : "Recovery Drill Failed";
  const badge = document.createElement("span");
  badge.textContent = result.canRecover ? "Ready" : "Needs files";
  header.append(title, badge);
  card.append(
    header,
    summaryGrid([
      [
        "Next",
        result.canRecover
          ? "Run recovery only when this device needs rewrap"
          : result.failureReason ?? "Select a valid non-cloud-only recovery set",
      ],
      [
        "Share bytes",
        result.recoveryShareBytesPrinted === false ? "not printed" : "check output",
      ],
    ]),
  );
  recoveryPlan.append(card);

  const files = document.createElement("ul");
  files.className = "recovery-file-list";
  for (const path of result.recoveryFiles ?? []) {
    const item = document.createElement("li");
    const label = document.createElement("strong");
    const value = document.createElement("span");
    label.textContent = "Checked";
    value.textContent = path;
    item.append(label, value);
    files.append(item);
  }
  recoveryPlan.append(files);
}

function recoveryHeader(title, rows) {
  const section = document.createElement("section");
  section.className = "recovery-summary";
  const heading = document.createElement("strong");
  heading.textContent = title;
  section.append(heading, summaryGrid(rows));
  return section;
}

function recoveryFileCard({ title, tone, destination, file }) {
  const card = document.createElement("article");
  card.className = "recovery-file-card";
  card.dataset.tone = tone;

  const header = document.createElement("div");
  header.className = "recovery-file-header";
  const name = document.createElement("strong");
  name.textContent = title;
  const group = document.createElement("span");
  group.textContent = file.group ? String(file.group).replaceAll("_", " ") : file.kind;
  header.append(name, group);

  const detailRows = [
    ["Destination", destination],
    ["Path", file.path],
    ["BLAKE3", file.blake3],
  ];
  if (file.kind === "share" || file.kind === "bundle") {
    detailRows.push(["Share bytes", file.shareBytesPrinted === false ? "not printed" : "-"]);
    if (file.kind === "bundle") {
      detailRows.push(["Vault data", "embedded"]);
    }
  } else if (file.containsSecretBytes === false) {
    detailRows.push(["Secret bytes", "not included"]);
  }
  const details = summaryGrid(detailRows);
  const placed = document.createElement("label");
  placed.className = "backup-check-row";
  const input = document.createElement("input");
  input.type = "checkbox";
  input.checked = isBackupPlaced(file);
  input.addEventListener("change", () => {
    setBackupPlaced(file, input.checked);
    refreshRecoveryPlacementUi();
  });
  const text = document.createElement("span");
  if (file.kind === "share" || file.kind === "bundle") {
    text.textContent = "Stored at destination";
  } else if (file.kind === "guide") {
    text.textContent = "Guide saved";
  } else {
    text.textContent = "Manifest saved";
  }
  placed.append(input, text);

  const actions = document.createElement("div");
  actions.className = "recovery-file-actions";
  const reveal = document.createElement("button");
  reveal.type = "button";
  reveal.textContent = "Show";
  reveal.addEventListener("click", () => {
    revealPath(file.path).catch(() => {});
  });
  actions.append(placed, reveal);

  card.append(header, details, actions);
  return card;
}

function recoveryShareOrder(shares) {
  const order = {
    cloud: 1,
    local_physical: 2,
    remote_physical: 3,
  };
  return [...shares].sort((left, right) => {
    const leftGroup = order[left.group] ?? 99;
    const rightGroup = order[right.group] ?? 99;
    if (leftGroup !== rightGroup) {
      return leftGroup - rightGroup;
    }
    return String(left.member ?? "").localeCompare(String(right.member ?? ""));
  });
}

function checkedRecoveryShares(shares, group) {
  return shares.filter((share) => share.group === group && isBackupPlaced(share));
}

function recoveryShareRole(file) {
  return recoveryArtifactRole(file);
}

function recoveryArtifactRole(file) {
  const member = String(file.member ?? "").toLowerCase();
  const destination = String(file.destination ?? "").toLowerCase();
  if (member.includes("icloud")) {
    return "icloud";
  }
  if (member.includes("google")) {
    return "google";
  }
  if (destination.includes("icloud")) {
    return "icloud";
  }
  if (destination.includes("google")) {
    return "google";
  }
  if (file.group === "local_physical") {
    return "local";
  }
  if (file.group === "remote_physical") {
    return "remote";
  }
  return file.group ?? file.kind ?? "unknown";
}

function recoveryArtifactKindLabel(file) {
  if (file.kind === "bundle") {
    return "backup file";
  }
  if (file.kind === "share") {
    return "backup file";
  }
  return file.kind ?? "file";
}

function recoverableRecoveryFileBucketsFromChecked(shares) {
  const placed = shares.filter(isBackupPlaced);
  const cloud = placed.filter((share) => share.group === "cloud");
  const local = placed.find((share) => share.group === "local_physical");
  const remote = placed.find((share) => share.group === "remote_physical");
  const cloudComplete =
    cloud.some((share) => recoveryShareRole(share) === "icloud") &&
    cloud.some((share) => recoveryShareRole(share) === "google");
  if (cloudComplete && local) {
    return {
      cloud: recoveryShareOrder(cloud).map((share) => share.path),
      local: [local.path],
    };
  }
  if (cloudComplete && remote) {
    return {
      cloud: recoveryShareOrder(cloud).map((share) => share.path),
      local: [remote.path],
    };
  }
  if (local && remote) {
    return {
      cloud: [],
      local: [local.path, remote.path],
    };
  }
  return { cloud: [], local: [] };
}

function missingRecommendedBackup(shares) {
  const missing = shares.filter((share) => !isBackupPlaced(share));
  if (missing.length === 0) {
    return null;
  }
  const next = recoveryShareOrder(missing)[0];
  return `Still place ${next.member ?? "remaining backup file"}`;
}

function backupPlacementKey(file) {
  return `${file.kind ?? "file"}:${file.blake3 ?? file.path ?? file.member ?? "unknown"}`;
}

function isBackupPlaced(file) {
  return backupPlacementState[backupPlacementKey(file)] === true;
}

function setBackupPlaced(file, placed) {
  const key = backupPlacementKey(file);
  if (placed) {
    backupPlacementState[key] = true;
  } else {
    delete backupPlacementState[key];
  }
  saveBackupPlacementState();
}

function refreshRecoveryPlacementUi() {
  if (latestRecoveryBackupOutcome) {
    renderRecoveryPanel();
    const selected = recoverableRecoveryFileBucketsFromChecked(currentRecoveryShares());
    if (recoveryFilesFromBuckets(selected).length > 0) {
      setRecoveryFileBuckets(selected);
      renderRecoveryInputStatus();
    }
  }
}

function fileNameFromPath(path) {
  if (!path) {
    return "";
  }
  return String(path).split(/[\\/]/).filter(Boolean).pop() ?? "";
}

function uniquePaths(paths) {
  const seen = new Set();
  const result = [];
  for (const path of paths ?? []) {
    const normalized = String(path ?? "").trim();
    if (normalized && !seen.has(normalized)) {
      seen.add(normalized);
      result.push(normalized);
    }
  }
  return result;
}

function activeRecoveryScheme() {
  return RECOVERY_SCHEMES[activeRecoverySchemeKey] ?? RECOVERY_SCHEMES.cloudPhysical;
}

function recoverySchemeSlots(schemeKey = activeRecoverySchemeKey) {
  return RECOVERY_SCHEMES[schemeKey]?.slots ?? RECOVERY_SCHEMES.cloudPhysical.slots;
}

function recoveryFilesFromSlots(
  slotFiles = selectedRecoverySlotFiles,
  schemeKey = activeRecoverySchemeKey,
) {
  return uniquePaths(recoverySchemeSlots(schemeKey).map((slot) => slotFiles[slot.key]).filter(Boolean));
}

function recoveryFilesFromBuckets({
  cloud = selectedCloudRecoveryFiles,
  local = selectedLocalRecoveryFiles,
} = {}) {
  return uniquePaths([...cloud, ...local]);
}

function selectedRecoveryFiles() {
  const slotFiles = recoveryFilesFromSlots();
  return slotFiles.length > 0 ? slotFiles : parsePathList(recoveryFilePaths.value);
}

function recoveryVaultSourcePath(files = selectedRecoveryFiles()) {
  return files[0] ?? "";
}

function syncRecoveryBucketsFromSlots() {
  const cloud = [];
  const local = [];
  for (const slot of recoverySchemeSlots()) {
    const path = selectedRecoverySlotFiles[slot.key];
    if (!path) {
      continue;
    }
    if (slot.bucket === "cloud") {
      cloud.push(path);
    } else {
      local.push(path);
    }
  }
  selectedCloudRecoveryFiles = uniquePaths(cloud);
  const cloudSet = new Set(selectedCloudRecoveryFiles);
  selectedLocalRecoveryFiles = uniquePaths(local).filter((path) => !cloudSet.has(path));
}

function syncRecoveryFilePathsFromBuckets() {
  const slotFiles = recoveryFilesFromSlots();
  const files = slotFiles.length > 0 ? slotFiles : recoveryFilesFromBuckets();
  recoveryFilePaths.value = files.join("\n");
  vaultBackupPath.value = recoveryVaultSourcePath(files);
}

function setRecoveryFileBuckets(
  { cloud = selectedCloudRecoveryFiles, local = selectedLocalRecoveryFiles } = {},
  { resetOutcomes = true } = {},
) {
  const normalizedCloud = uniquePaths(cloud);
  const normalizedLocal = uniquePaths(local).filter((path) => !normalizedCloud.includes(path));
  activeRecoverySchemeKey = normalizedCloud.length > 0 ? "cloudPhysical" : "physicalPair";
  selectedRecoverySlotFiles = recoverySlotFilesFromBuckets(
    { cloud: normalizedCloud, local: normalizedLocal },
    activeRecoverySchemeKey,
  );
  syncRecoveryBucketsFromSlots();
  syncRecoveryFilePathsFromBuckets();
  if (resetOutcomes) {
    markRecoverySelectionChanged();
  }
}

function recoverySlotFilesFromBuckets({ cloud = [], local = [] } = {}, schemeKey) {
  const slotFiles = {};
  if (schemeKey === "physicalPair") {
    const localPath = findRecoveryPathByRole(local, "local") ?? local[0] ?? "";
    const remotePath =
      findRecoveryPathByRole(local, "remote") ?? local.find((path) => path !== localPath) ?? "";
    if (localPath) {
      slotFiles.local = localPath;
    }
    if (remotePath) {
      slotFiles.remote = remotePath;
    }
    return slotFiles;
  }

  const icloudPath = findRecoveryPathByRole(cloud, "icloud") ?? cloud[0] ?? "";
  const googlePath =
    findRecoveryPathByRole(cloud, "google") ?? cloud.find((path) => path !== icloudPath) ?? "";
  const physicalPath =
    findRecoveryPathByRole(local, "local") ??
    findRecoveryPathByRole(local, "remote") ??
    local[0] ??
    "";
  if (icloudPath) {
    slotFiles.icloud = icloudPath;
  }
  if (googlePath) {
    slotFiles.google = googlePath;
  }
  if (physicalPath) {
    slotFiles.physical = physicalPath;
  }
  return slotFiles;
}

function findRecoveryPathByRole(paths, role) {
  return uniquePaths(paths).find((path) => recoveryIntrinsicFileRole(path).role === role);
}

function assignRecoveryFilesToBucket(bucket, paths) {
  if (bucket === "cloud") {
    setRecoveryFileBuckets({ cloud: paths, local: selectedLocalRecoveryFiles });
  } else {
    setRecoveryFileBuckets({ cloud: selectedCloudRecoveryFiles, local: paths });
  }
}

function assignRecoveryFileToSlot(slotKey, paths) {
  const slot = recoverySchemeSlots().find((candidate) => candidate.key === slotKey);
  if (!slot) {
    return;
  }
  const selected = recoveryPathForSlot(slot, paths);
  if (!selected) {
    return;
  }
  for (const key of Object.keys(selectedRecoverySlotFiles)) {
    if (selectedRecoverySlotFiles[key] === selected) {
      delete selectedRecoverySlotFiles[key];
    }
  }
  selectedRecoverySlotFiles = {
    ...selectedRecoverySlotFiles,
    [slot.key]: selected,
  };
  syncRecoveryBucketsFromSlots();
  syncRecoveryFilePathsFromBuckets();
  markRecoverySelectionChanged();
}

function recoveryPathForSlot(slot, paths) {
  const selected = uniquePaths(paths);
  if (selected.length === 0) {
    return "";
  }
  return selected.find((path) => recoverySlotAcceptsRole(slot, recoveryIntrinsicFileRole(path).role)) ?? selected[0];
}

function recoverySlotAcceptsRole(slot, role) {
  if (slot.key === "physical") {
    return role === "local" || role === "remote" || role === "physical_unknown";
  }
  return role === slot.role;
}

function setRecoveryScheme(schemeKey) {
  if (!RECOVERY_SCHEMES[schemeKey] || activeRecoverySchemeKey === schemeKey) {
    renderRecoveryInputStatus();
    return;
  }
  activeRecoverySchemeKey = schemeKey;
  syncRecoveryBucketsFromSlots();
  syncRecoveryFilePathsFromBuckets();
  markRecoverySelectionChanged();
  renderRecoveryInputStatus();
}

function clearRecoveryFileBuckets({ syncInput = true, resetOutcomes = true } = {}) {
  selectedCloudRecoveryFiles = [];
  selectedLocalRecoveryFiles = [];
  selectedRecoverySlotFiles = {};
  if (syncInput) {
    recoveryFilePaths.value = "";
    vaultBackupPath.value = "";
  }
  if (resetOutcomes) {
    markRecoverySelectionChanged();
  }
}

function recoverySelectedSlotForPath(path) {
  const normalized = String(path ?? "").trim();
  return recoverySchemeSlots().find((slot) => selectedRecoverySlotFiles[slot.key] === normalized);
}

function recoveryFileBucketForPath(path) {
  const slot = recoverySelectedSlotForPath(path);
  if (slot) {
    return slot.bucket;
  }
  if (selectedCloudRecoveryFiles.includes(path)) {
    return "cloud";
  }
  if (selectedLocalRecoveryFiles.includes(path)) {
    return "local";
  }
  return "manual";
}

function clearBackupPlacementStateForCurrentPlan() {
  for (const file of latestRecoveryBackupOutcome?.recoveryBackups?.files ?? []) {
    delete backupPlacementState[backupPlacementKey(file)];
  }
  saveBackupPlacementState();
}

function loadBackupPlacementState() {
  try {
    const raw = window.localStorage?.getItem(BACKUP_PLACEMENT_STORAGE_KEY);
    if (!raw) {
      return {};
    }
    const parsed = JSON.parse(raw);
    return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? parsed : {};
  } catch {
    return {};
  }
}

function saveBackupPlacementState() {
  try {
    window.localStorage?.setItem(
      BACKUP_PLACEMENT_STORAGE_KEY,
      JSON.stringify(backupPlacementState),
    );
  } catch {
    // Checklist persistence is best-effort local UI state.
  }
}

function prefillRecoveryFilesFromBackup() {
  vaultBackupPath.value = recoveryVaultSourcePath();
}

function renderRecoveryInputStatus() {
  const state = recoveryInputState();
  const backupPath = recoveryVaultSourcePath(state.files);
  vaultBackupPath.value = backupPath;
  if (recoverySetSummary) {
    recoverySetSummary.textContent = state.summary;
    recoverySetSummary.dataset.tone = state.tone;
  }
  if (recoverySetPolicy) {
    recoverySetPolicy.textContent = state.policy;
    recoverySetPolicy.dataset.tone = state.policyTone;
  }
  renderRestoreSchemeCards();
  renderRestoreFileSlots();
  renderRestoreSelectedFiles(state);
  setRestoreStep(
    restoreStepFiles,
    state.files.length > 0
      ? state.tone === "good"
        ? "good"
        : state.tone === "bad"
          ? "bad"
          : "active"
      : "active",
    state.summary,
  );
  if (restoreWriteSummary) {
    restoreWriteSummary.textContent = latestRecoveryRecoverOutcome
      ? "Wallet restored on this Mac"
      : state.files.length === 0
        ? "Choose backup files first"
        : !state.canAttemptRestore
          ? "Add the required backup files"
        : recoverOverwrite.checked
          ? "Ready to restore"
          : "Confirm device replacement";
  }
  setRestoreStep(
    restoreStepWrite,
    latestRecoveryRecoverOutcome
      ? "good"
      : state.canAttemptRestore
        ? recoverOverwrite.checked
          ? "active"
          : "warn"
        : "locked",
    restoreWriteSummary?.textContent,
  );
  recoverVaultButton.disabled = recoveringVault || !state.canAttemptRestore || !recoverOverwrite.checked;
  updateRecoveryInputControls();
}

function setRestoreStep(element, state, summary) {
  if (!element) {
    return;
  }
  element.dataset.state = state;
  if (summary) {
    element.setAttribute("aria-label", summary);
  }
}

function renderRestoreSelectedFiles(state) {
  if (!restoreSelectedFiles) {
    return;
  }
  restoreSelectedFiles.replaceChildren();
  if (state.files.length === 0 || state.usesSlots) {
    restoreSelectedFiles.hidden = true;
    return;
  }
  restoreSelectedFiles.hidden = false;
  for (const path of state.files) {
    const role = recoverySelectedFileRole(path);
    restoreSelectedFiles.append(
      restoreFileChip({
        label: restoreRecoveryFileLabel(path, role),
        detail: fileNameFromPath(path) || path,
        tone: role.known ? "good" : "warn",
      }),
    );
  }
}

function renderRestoreSchemeCards() {
  for (const card of restoreSchemeCards) {
    const selected = card.dataset.recoveryScheme === activeRecoverySchemeKey;
    card.dataset.state = selected ? "selected" : "idle";
    card.setAttribute("aria-pressed", selected ? "true" : "false");
  }
}

function renderRestoreFileSlots() {
  if (!restoreFileSlots) {
    return;
  }
  const slots = recoverySchemeSlots();
  restoreFileSlots.dataset.count = String(slots.length);
  restoreFileSlots.replaceChildren();
  for (const slot of slots) {
    const path = selectedRecoverySlotFiles[slot.key] ?? "";
    const button = document.createElement("button");
    button.type = "button";
    button.className = "restore-file-slot";
    button.dataset.recoverySlot = slot.key;
    button.dataset.state = path ? "selected" : "empty";
    button.title = path || slot.detail;

    const label = document.createElement("span");
    label.textContent = slot.label;
    const value = document.createElement("strong");
    value.textContent = path ? fileNameFromPath(path) || "Selected file" : "Choose file";
    const detail = document.createElement("small");
    detail.textContent = path ? slot.detail : slot.empty;

    button.append(label, value, detail);
    restoreFileSlots.append(button);
  }
}

function restoreFileChip({ label, detail, tone }) {
  const chip = document.createElement("article");
  chip.className = "restore-file-chip";
  chip.dataset.tone = tone;
  const title = document.createElement("strong");
  title.textContent = label;
  const body = document.createElement("span");
  body.textContent = detail;
  chip.append(title, body);
  return chip;
}

function restoreRecoveryFileLabel(_path, role = recoverySelectedFileRole(_path)) {
  if (role.role === "icloud") {
    return "iCloud backup file";
  }
  if (role.role === "google") {
    return "Google backup file";
  }
  if (role.role === "local") {
    return "Local backup file";
  }
  if (role.role === "remote") {
    return "Off-site backup file";
  }
  if (role.role === "cloud_unknown") {
    return "Cloud backup file";
  }
  if (role.role === "physical_unknown") {
    return "Physical backup file";
  }
  return "Backup file";
}

function recoveryIntrinsicFileRole(path) {
  const normalized = String(path ?? "").trim();
  const name = fileNameFromPath(normalized).toLowerCase();
  const planFile = currentRecoveryShares().find((file) => {
    const filePath = String(file.path ?? "");
    return filePath === normalized || fileNameFromPath(filePath).toLowerCase() === name;
  });
  if (planFile) {
    return { role: recoveryArtifactRole(planFile), known: true };
  }
  if (name === "backup-01.dat") {
    return { role: "icloud", known: true };
  }
  if (name === "backup-02.dat") {
    return { role: "google", known: true };
  }
  if (name === "backup-03.dat") {
    return { role: "local", known: true };
  }
  if (name === "backup-04.dat") {
    return { role: "remote", known: true };
  }
  return { role: "unknown", known: false };
}

function recoverySelectedFileRole(path) {
  const normalized = String(path ?? "").trim();
  const intrinsic = recoveryIntrinsicFileRole(normalized);
  if (intrinsic.known) {
    return intrinsic;
  }
  const slot = recoverySelectedSlotForPath(normalized);
  if (slot) {
    return { role: slot.role, known: false };
  }
  const bucket = recoveryFileBucketForPath(normalized);
  if (bucket === "cloud") {
    return { role: "cloud_unknown", known: false };
  }
  if (bucket === "local") {
    return { role: "physical_unknown", known: false };
  }
  return { role: "unknown", known: false };
}

function recoveryKnownRoleSet(files) {
  return new Set(files.map((path) => recoverySelectedFileRole(path).role));
}

function recoverySetKnownSufficient(files) {
  const roles = recoveryKnownRoleSet(files);
  const cloudPair = roles.has("icloud") && roles.has("google");
  const hasPhysical = roles.has("local") || roles.has("remote");
  const physicalPair = roles.has("local") && roles.has("remote");
  return (cloudPair && hasPhysical) || physicalPair;
}

function recoveryHasUnknownFiles(files) {
  return files.some((path) => !recoverySelectedFileRole(path).known);
}

function recoveryDrillUserMessage(result) {
  if (result.canRecover) {
    return "These files are enough to recover the wallet.";
  }
  const reason = String(result.failureReason ?? "").toLowerCase();
  if (reason.includes("cloud") || reason.includes("physical")) {
    return "These files are not enough yet. Add a Local backup file and check again.";
  }
  return "These files are not enough yet. Choose another backup file and check again.";
}

function recoveryInputState() {
  const slotFiles = recoveryFilesFromSlots();
  const usesSlots = slotFiles.length > 0;
  const files = usesSlots ? slotFiles : parsePathList(recoveryFilePaths.value);
  const source = recoverySourceSlotState(files);
  const knownSufficient = recoverySetKnownSufficient(files);
  const hasUnknown = recoveryHasUnknownFiles(files);
  if (files.length === 0) {
    return {
      files,
      source,
      usesSlots: false,
      canAttemptRestore: false,
      tone: "idle",
      policyTone: "idle",
      summary: "No backup files selected",
      policy: activeRecoveryScheme().summary,
    };
  }

  if (usesSlots) {
    const slotState = recoverySchemeSlotState();
    return {
      files,
      source,
      usesSlots: true,
      canAttemptRestore: slotState.complete,
      tone: slotState.complete ? "good" : "bad",
      policyTone: slotState.complete ? "good" : "bad",
      summary: slotState.summary,
      policy: slotState.policy,
    };
  }

  const canAttemptRestore = knownSufficient || hasUnknown;
  return {
    files,
    source,
    usesSlots: false,
    canAttemptRestore,
    tone: knownSufficient ? "good" : hasUnknown ? "warn" : "bad",
    policyTone: knownSufficient ? "good" : hasUnknown ? "warn" : "bad",
    summary: recoverySlotSummary(source, files.length),
    policy: recoverySlotPolicy(source, files),
  };
}

function recoverySchemeSlotState() {
  const slots = recoverySchemeSlots();
  const missing = slots.filter((slot) => !selectedRecoverySlotFiles[slot.key]);
  const selectedCount = slots.length - missing.length;
  const complete = missing.length === 0;
  if (complete) {
    return {
      complete: true,
      summary:
        activeRecoverySchemeKey === "physicalPair"
          ? "Local + off-site selected"
          : "iCloud + Google + physical selected",
      policy: "Ready to restore. FRAMKey will verify the files before writing.",
    };
  }
  const missingLabels = missing.map((slot) => slot.label).join(", ");
  return {
    complete: false,
    summary: `${selectedCount} of ${slots.length} backup files selected`,
    policy: `Add ${missingLabels}.`,
  };
}

function recoverySourceSlotState(files) {
  const roles = files.map((path) => recoverySelectedFileRole(path));
  const hasIcloud = roles.some((item) => item.role === "icloud");
  const hasGoogle = roles.some((item) => item.role === "google");
  const hasLocal = roles.some((item) => item.role === "local");
  const hasRemote = roles.some((item) => item.role === "remote");
  const cloudUnknown = roles.filter((item) => item.role === "cloud_unknown").length;
  const physicalUnknown = roles.filter((item) => item.role === "physical_unknown").length;
  return {
    cloud: {
      count: Number(hasIcloud) + Number(hasGoogle) + cloudUnknown,
      complete: hasIcloud && hasGoogle,
      hasIcloud,
      hasGoogle,
      unknown: cloudUnknown,
    },
    physical: {
      count: Number(hasLocal) + Number(hasRemote) + physicalUnknown,
      complete: hasLocal && hasRemote,
      hasLocal,
      hasRemote,
      unknown: physicalUnknown,
    },
    unknownCount: roles.filter((item) => !item.known).length,
  };
}

function recoverySlotSummary(source, fileCount) {
  if (source.cloud.complete && source.physical.count > 0) {
    return `${fileCount} files selected: cloud pair + physical`;
  }
  if (source.physical.complete) {
    return `${fileCount} files selected: local + off-site`;
  }
  if (source.cloud.count > 0) {
    return `${source.cloud.count} cloud file${source.cloud.count === 1 ? "" : "s"} selected`;
  }
  if (source.physical.count > 0) {
    return `${source.physical.count} physical file${source.physical.count === 1 ? "" : "s"} selected`;
  }
  return `${fileCount} file${fileCount === 1 ? "" : "s"} selected`;
}

function recoverySlotPolicy(source, files) {
  if (recoverySetKnownSufficient(files)) {
    if (source.physical.complete && !source.cloud.complete) {
      return "Ready: local and off-site physical files are enough.";
    }
    return "Ready: both cloud files plus one physical file.";
  }
  if (source.unknownCount > 0) {
    return "FRAMKey will verify these files during restore.";
  }
  if (source.cloud.complete && source.physical.count === 0) {
    return "Need one physical file too: backup-03.dat or backup-04.dat.";
  }
  if (source.cloud.count === 1 && source.physical.count > 0) {
    return "Need the second cloud file, or use both physical files instead.";
  }
  if (source.cloud.count === 1) {
    return "Need both cloud files plus one physical file, or both physical files.";
  }
  if (source.physical.count === 1) {
    return "Add both cloud files, or add the other physical file.";
  }
  if (source.cloud.count > 0 || source.physical.count > 0) {
    return "These files are not enough yet.";
  }
  return "Choose the backup files you have. FRAMKey will verify them during restore.";
}

function updateRecoveryInputControls() {
  clearRecoveryFilesButton.disabled = selectedRecoveryFiles().length === 0;
}

function emptyRecoveryBuckets() {
  return { cloud: [], local: [] };
}

function markRecoverySelectionChanged() {
  latestRecoveryDrillOutcome = null;
  latestRecoveryRecoverOutcome = null;
}

function recoveryTone(group) {
  if (group === "cloud") {
    return "cloud";
  }
  if (group === "local_physical") {
    return "local";
  }
  if (group === "remote_physical") {
    return "remote";
  }
  return "neutral";
}

function recoveryDestination(file) {
  const member = String(file.member ?? "").toLowerCase();
  if (member.includes("icloud")) {
    return "Upload to iCloud Drive";
  }
  if (member.includes("google")) {
    return "Upload to Google Drive";
  }
  if (member.includes("local")) {
    return "Copy to local physical storage";
  }
  if (member.includes("off-site")) {
    return "Store off-site away from this Mac and GBA card";
  }
  return "Store according to this backup label";
}

function recommendedRecoveryFileBuckets(result) {
  const files = result.recoveryBackups?.files ?? [];
  const cloud = files
    .filter((file) => file.kind === "bundle" && file.group === "cloud")
    .map((file) => file.path);
  const local = files.find((file) => file.kind === "bundle" && file.group === "local_physical");
  return {
    cloud,
    local: local?.path ? [local.path] : [],
  };
}

function recommendedVaultBackupFile(result) {
  const files = result.recoveryBackups?.files ?? [];
  const backup = files.find((file) => file.kind === "bundle" && file.path);
  return backup?.path ?? "";
}

function renderReviewRequest(request) {
  const item = document.createElement("article");
  item.className = "review-item";
  item.dataset.status = request.status ?? "pending";
  if (request.id) {
    item.dataset.reviewId = request.id;
  }

  const header = document.createElement("div");
  header.className = "review-item-header";

  const title = document.createElement("div");
  title.className = "review-title";
  const method = document.createElement("strong");
  method.textContent = reviewIntentTitle(request);
  const meta = document.createElement("span");
  meta.textContent = `${request.origin ?? "unknown origin"} · ${request.method ?? "wallet request"} · expires ${formatTime(
    request.expiresAtUnixMs,
  )}`;
  title.append(method, meta);

  const status = document.createElement("span");
  status.className = "review-status";
  status.textContent = String(request.status ?? "blocked").replaceAll("_", " ");
  status.dataset.status = request.status ?? "pending";

  header.append(title, status);

  const synopsis = renderReviewSynopsis(request);

  const rawSummary = document.createElement("details");
  rawSummary.className = "review-params";
  const rawSummaryLabel = document.createElement("summary");
  rawSummaryLabel.textContent = "Technical summary";
  const rawSummaryBody = document.createElement("pre");
  rawSummaryBody.textContent = JSON.stringify(request.summary ?? {}, null, 2);
  rawSummary.append(rawSummaryLabel, rawSummaryBody);

  const simulation = renderSimulationReport(
    request.summary?.simulation,
    request.summary?.policy,
    request.summary?.assetContext,
  );

  const params = document.createElement("details");
  params.className = "review-params";
  const paramsSummary = document.createElement("summary");
  paramsSummary.textContent = "Request data";
  const paramsBody = document.createElement("pre");
  paramsBody.textContent = JSON.stringify(request.paramsPreview ?? null, null, 2);
  params.append(paramsSummary, paramsBody);

  const footer = document.createElement("div");
  footer.className = "review-footer";
  const reason = document.createElement("span");
  reason.textContent = formatReviewReason(request);

  const reviewActions = document.createElement("div");
  reviewActions.className = "review-actions";
  if (request.status === "pending") {
    const approveAction = approveActionForRequest(request);
    const approve = document.createElement("button");
    approve.type = "button";
    approve.textContent = approveAction.label;
    approve.disabled = approveAction.disabled;
    if (approveAction.disabledReason) {
      approve.title = approveAction.disabledReason;
    }
    if (approveAction.tone) {
      approve.dataset.tone = approveAction.tone;
    }
    if (!approveAction.disabled) {
      approve.addEventListener("click", () => {
        decideReviewRequest(request, approveAction.decision).catch(() => {});
      });
    }

    const reject = document.createElement("button");
    reject.type = "button";
    reject.textContent = "Reject";
    reject.addEventListener("click", () => {
      decideReviewRequest(request, "reject").catch(() => {});
    });

    reviewActions.append(approve, reject);
  }

  const dismiss = document.createElement("button");
  dismiss.type = "button";
  dismiss.textContent = "Dismiss";
  dismiss.addEventListener("click", () => {
    dismissReviewRequest(request.id).catch(() => {});
  });
  reviewActions.append(dismiss);
  footer.append(reason, reviewActions);

  item.append(header, synopsis);
  if (simulation) {
    item.append(simulation);
  }
  item.append(rawSummary, params, footer);
  return item;
}

function renderReviewSynopsis(request) {
  const summary = request.summary ?? {};
  const card = document.createElement("section");
  card.className = "review-synopsis";

  const intent = document.createElement("div");
  intent.className = "review-intent";
  const heading = document.createElement("strong");
  heading.textContent = reviewIntentTitle(request);
  const subtitle = document.createElement("span");
  subtitle.textContent = reviewIntentSubtitle(request);
  intent.append(heading, subtitle);
  card.append(intent);

  if (request.kind === "transaction") {
    const guidance = renderTransactionGuidance(summary.guidance);
    if (guidance) {
      card.append(guidance);
    }
    card.append(renderTransactionRisk(summary.risk, summary.policy));
    const trust = renderTransactionTrust(summary.trust);
    if (trust) {
      card.append(trust);
    }
    const riskDetails = renderTransactionRiskDetails(summary);
    if (riskDetails) {
      card.append(riskDetails);
    }
    const impact = renderTransactionImpact(summary.impact);
    if (impact) {
      card.append(impact);
    }
    const rows = [
      ["From", shortAddress(summary.from)],
      ["To", shortAddress(summary.to)],
      ["Value", formatTransactionValue(summary)],
      ["Asset", transactionAssetContextLabel(summary.assetContext)],
      ["Protocol", transactionProtocolLabel(summary.simulation)],
      ["Call", transactionCallLabel(summary.simulation)],
      ["Network", summary.chainId ?? "-"],
      ["Gas", valueOrDash(summary.gas)],
      ["Nonce", valueOrDash(summary.nonce)],
      ["Data", `${summary.dataBytes ?? 0} bytes`],
    ];
    card.append(summaryGrid(rows));
    return card;
  }

  if (request.kind === "account_connection") {
    card.append(
      summaryGrid([
        ["Permission", summary.permission ?? "eth_accounts"],
        ["Requested", (summary.requestedPermissions ?? ["eth_accounts"]).join(", ")],
        ["Decision", summary.decision ?? "requires approval"],
      ]),
    );
    return card;
  }

  if (request.kind === "network_switch") {
    card.append(
      summaryGrid([
        ["Intent", networkManagementIntentLabel(summary.intent)],
        ["Current", summary.currentChainId ?? "-"],
        ["Requested", summary.requestedChainId ?? "-"],
        ["RPC", networkManagementRpcLabel(summary)],
        ["Decision", summary.decision ?? "requires approval"],
      ]),
    );
    return card;
  }

  if (request.kind === "watch_asset") {
    card.append(
      summaryGrid([
        ["Asset", `${valueOrDash(summary.symbol)} · ${valueOrDash(summary.assetType)}`],
        ["Contract", shortAddress(summary.contractAddress)],
        ["Decimals", summary.decimals ?? "-"],
        ["Network", summary.chainId ?? "-"],
        ["Source", summary.source === "dapp_request" ? "dApp request" : valueOrDash(summary.source)],
        ["Decision", summary.decision ?? "requires approval"],
      ]),
    );
    return card;
  }

  if (request.kind === "personal_sign") {
    const message = summary.message ?? {};
    card.append(
      summaryGrid([
        ["Account", shortAddress(summary.account)],
        ["Message", message.preview ?? message.utf8Preview ?? "-"],
        ["Encoding", message.encoding ?? "-"],
        ["Size", message.bytes != null ? `${message.bytes} bytes` : `${message.chars ?? 0} chars`],
      ]),
    );
    return card;
  }

  if (request.kind === "typed_data") {
    const typedData = summary.typedData ?? {};
    const permit = typedData.permit ?? {};
    card.append(
      summaryGrid([
        ["Account", shortAddress(summary.account)],
        ["Intent", typedPermitIntentLabel(typedData.intent ?? typedData.primaryType)],
        ["Domain", typedDataDomainLabel(typedData.domain)],
        ["Owner", shortAddress(permit.owner)],
        ["Spender", shortAddress(permit.spender)],
        ["Token", typedPermitTokenLabel(permit)],
        ["Amount", typedPermitAmountLabel(permit)],
        ["Deadline", typedPermitDeadlineLabel(permit)],
        ["Decision", typedDataSigningAllowed(request) ? "requires trusted approval" : "blocked before signing"],
      ]),
    );
    return card;
  }

  card.append(
    summaryGrid([
      ["Account", shortAddress(summary.account)],
      ["Intent", summary.intent ?? request.method ?? "-"],
      ["Decision", summary.decision ?? "dry_run"],
    ]),
  );
  return card;
}

function reviewIntentTitle(request) {
  if (request.kind === "account_connection") {
    return "Connect Wallet";
  }
  if (request.kind === "network_switch") {
    return request.summary?.intent === "add_network" ? "Add Network" : "Change Network";
  }
  if (request.kind === "watch_asset") {
    return "Add Token";
  }
  if (request.kind === "transaction") {
    return transactionRequestTitle(request.summary);
  }
  if (request.kind === "personal_sign") {
    return "Sign Message";
  }
  if (request.kind === "typed_data") {
    return "Approve Token Permission";
  }
  return request.method ?? "Blocked Request";
}

function transactionRequestTitle(summary) {
  const impact = summary?.impact;
  const approvalCount = impact?.approvalCount ?? 0;
  const transferCount = impact?.transferCount ?? 0;
  const protocol = transactionProtocolLabel(summary?.simulation);
  if (approvalCount > 0 && transferCount === 0) {
    return "Token Approval";
  }
  if (approvalCount > 0 && transferCount > 0) {
    return "Swap With Approval";
  }
  if (transferCount > 0) {
    return protocol !== "-" ? `${protocol} Transaction` : "Send Transaction";
  }
  if (protocol !== "-") {
    return `${protocol} Transaction`;
  }
  return "Review Transaction";
}

function reviewIntentSubtitle(request) {
  const origin = request.origin ?? "unknown origin";
  if (request.kind === "transaction") {
    const policy = request.summary?.policy?.decision ?? "policy pending";
    return `${origin} · ${String(policy).replaceAll("_", " ")}`;
  }
  if (request.kind === "typed_data") {
    const intent = request.summary?.typedData?.intent ?? "blocked before signing";
    return `${origin} · ${String(intent).replaceAll("_", " ")}`;
  }
  if (request.kind === "network_switch") {
    const action = request.summary?.intent === "add_network" ? "add" : "switch";
    return `${origin} · ${action} ${request.summary?.currentChainId ?? "-"} -> ${
      request.summary?.requestedChainId ?? "-"
    }`;
  }
  if (request.kind === "watch_asset") {
    return `${origin} · ${valueOrDash(request.summary?.symbol)} ${shortAddress(
      request.summary?.contractAddress,
    )}`;
  }
  return origin;
}

function networkManagementIntentLabel(intent) {
  if (intent === "add_network") {
    return "Add supported network";
  }
  if (intent === "switch_network") {
    return "Switch active network";
  }
  return valueOrDash(intent);
}

function networkManagementRpcLabel(summary) {
  const source = summary?.rpcSource;
  if (source === "trusted_alchemy_only") {
    return "FRAMKey Alchemy endpoint; dApp RPC ignored";
  }
  if (source === "trusted_alchemy_session") {
    return "FRAMKey Alchemy session";
  }
  return valueOrDash(source);
}

function renderTransactionRisk(riskSummary, policy) {
  const risk = document.createElement("div");
  risk.className = "risk-strip";
  risk.dataset.tone = transactionRiskTone(riskSummary, policy);
  if (riskSummary?.title) {
    const action = transactionRiskActionLabel(riskSummary.action, policy);
    risk.textContent = `${riskSummary.title} · ${action}`;
  } else if (policy?.canSign) {
    risk.textContent = "Ready for ordinary approval";
  } else if (policy?.overrideAllowed) {
    risk.textContent = "High-risk confirmation required";
  } else {
    risk.textContent = `Blocked by policy: ${transactionPolicyDecisionLabel(policy)}`;
  }
  return risk;
}

function renderTransactionGuidance(guidance) {
  if (!guidance || typeof guidance !== "object") {
    return null;
  }
  const section = document.createElement("section");
  section.className = "transaction-guidance";
  section.dataset.tone = guidance.tone ?? guidanceTone(guidance);

  const header = document.createElement("div");
  header.className = "transaction-guidance-head";
  const title = document.createElement("strong");
  title.textContent = guidance.title ?? "Review transaction";
  const action = document.createElement("span");
  action.textContent = guidance.primaryAction ?? "Review";
  header.append(title, action);

  const message = document.createElement("p");
  message.textContent = guidance.message ?? "-";
  const next = document.createElement("p");
  next.className = "transaction-guidance-next";
  next.textContent = guidance.nextStep ?? "Review the transaction details before deciding.";
  section.append(header, message, next);
  return section;
}

function renderTransactionRiskDetails(summary) {
  const policy = summary?.policy;
  if (!policy || typeof policy !== "object") {
    return null;
  }
  const simulation = summary?.simulation ?? {};
  const risk = summary?.risk ?? null;
  const blockers = Array.isArray(policy.blockers) ? policy.blockers : [];
  const warnings = Array.isArray(simulation.warnings) ? simulation.warnings : [];
  const extraWarnings = warnings.filter((warning) => !policyBlockerCoversWarning(blockers, warning));
  const riskReasons = Array.isArray(risk?.reasons) ? risk.reasons : [];

  const section = document.createElement("section");
  section.className = "risk-details";
  section.dataset.tone = transactionRiskTone(risk, policy);
  section.append(
    summaryGrid([
      ["Risk", transactionRiskLevel(risk, policy)],
      ["Simulation", transactionSimulationLabel(simulation)],
      ["Approval", transactionRiskActionLabel(risk?.action, policy)],
      ["Broadcast", policy.canBroadcast ? "allowed after signing" : "not allowed"],
      ["Reasons", riskReasons.length > 0 ? `${riskReasons.length} review item(s)` : "none"],
    ]),
  );

  if (risk?.message) {
    const note = document.createElement("p");
    note.className = "risk-note";
    note.textContent = risk.message;
    section.append(note);
  }

  if (riskReasons.length > 0) {
    section.append(
      riskItemList(
        riskReasons.slice(0, 6).map((reason) => ({
          tone: riskReasonTone(reason),
          title: reason.title ?? policyBlockerTitle(reason.code),
          code: `${reason.source ?? "risk"} · ${reason.code ?? "review"}`,
          detail: reason.message ?? "review reason",
        })),
      ),
    );
    return section;
  }

  if (blockers.length > 0) {
    section.append(
      riskItemList(
        blockers.map((blocker) => ({
          tone: blocker.overrideable ? "warn" : "bad",
          title: policyBlockerTitle(blocker.code),
          code: blocker.code,
          detail: `${blocker.overrideable ? "High-risk override allowed" : "Cannot sign"} · ${
            blocker.message ?? "policy blocker"
          }`,
        })),
      ),
    );
  }

  if (extraWarnings.length > 0) {
    section.append(
      riskItemList(
        extraWarnings.slice(0, 4).map((warning) => ({
          tone: warning.severity === "error" ? "bad" : "warn",
          title: simulationWarningTitle(warning.code),
          code: warning.code,
          detail: warning.message ?? "simulation warning",
        })),
      ),
    );
  }

  return section;
}

function renderTransactionImpact(impact) {
  if (!impact || typeof impact !== "object") {
    return null;
  }
  const section = document.createElement("section");
  section.className = "impact-summary";

  const head = document.createElement("div");
  head.className = "impact-summary-head";
  const title = document.createElement("strong");
  title.textContent = impact.title ?? "Transaction impact";
  const meta = document.createElement("span");
  const live = impact.liveSimulated ? "live simulation" : "local decode";
  meta.textContent = `${live} · ${impact.transferCount ?? 0} transfer(s) · ${impact.approvalCount ?? 0} approval(s)`;
  head.append(title, meta);
  section.append(head);

  if (Array.isArray(impact.items) && impact.items.length > 0) {
    section.append(
      riskItemList(
        impact.items.slice(0, 6).map((item) => ({
          tone: impactItemTone(item),
          title: item.title ?? "Impact",
          code: String(item.kind ?? "impact").replaceAll("_", " "),
          detail: item.message ?? "-",
        })),
      ),
    );
  }
  return section;
}

function renderTransactionTrust(trust) {
  if (!trust || typeof trust !== "object") {
    return null;
  }
  const section = document.createElement("section");
  section.className = "trust-summary";
  section.dataset.tone = transactionTrustTone(trust);

  const head = document.createElement("div");
  head.className = "trust-summary-head";
  const title = document.createElement("strong");
  title.textContent = trust.title ?? "Counterparty trust";
  const meta = document.createElement("span");
  meta.textContent = `${trust.knownCount ?? 0} known · ${trust.unknownCount ?? 0} unknown`;
  head.append(title, meta);
  section.append(head);

  if (Array.isArray(trust.items) && trust.items.length > 0) {
    section.append(
      riskItemList(
        trust.items.slice(0, 6).map((item) => ({
          tone: trustItemTone(item),
          title: transactionTrustItemTitle(item),
          code: String(item.role ?? "counterparty").replaceAll("_", " "),
          detail: item.message ?? shortAddress(item.address) ?? "-",
        })),
      ),
    );
  }
  return section;
}

function riskItemList(items) {
  const list = document.createElement("ul");
  list.className = "risk-items";
  for (const item of items) {
    const row = document.createElement("li");
    row.dataset.tone = item.tone ?? "warn";
    const title = document.createElement("strong");
    title.textContent = item.title;
    const detail = document.createElement("span");
    detail.textContent = item.detail;
    const code = document.createElement("small");
    code.textContent = item.code ?? "policy";
    row.append(title, detail, code);
    list.append(row);
  }
  return list;
}

function transactionPolicyTone(policy) {
  if (policy?.canSign) {
    return "good";
  }
  if (policy?.overrideAllowed) {
    return "warn";
  }
  return "bad";
}

function transactionRiskTone(risk, policy) {
  const tones = {
    low: "good",
    caution: "warn",
    high: "warn",
    blocked: "bad",
  };
  return tones[risk?.level] ?? transactionPolicyTone(policy);
}

function guidanceTone(guidance) {
  if (guidance?.blocked) {
    return "bad";
  }
  if (guidance?.requiresHighRisk) {
    return "warn";
  }
  if (guidance?.canApprove) {
    return "good";
  }
  return "warn";
}

function transactionRiskLevel(risk, policy) {
  const labels = {
    low: "Low",
    caution: "Caution",
    high: "High",
    blocked: "Blocked",
  };
  if (risk?.level && labels[risk.level]) {
    return labels[risk.level];
  }
  if (policy?.canSign) {
    return "Low";
  }
  if (policy?.overrideAllowed) {
    return "High";
  }
  return "Blocked";
}

function transactionRiskActionLabel(action, policy) {
  const labels = {
    ordinary_approval: "Ordinary approval",
    high_risk_approval: "High-risk approval",
    blocked: "Blocked",
  };
  if (action && labels[action]) {
    return labels[action];
  }
  if (policy?.canSign) {
    return "Ordinary approval";
  }
  if (policy?.overrideAllowed) {
    return "High-risk approval";
  }
  return "Blocked";
}

function riskReasonTone(reason) {
  if (reason?.severity === "error") {
    return "bad";
  }
  if (reason?.severity === "info") {
    return "good";
  }
  return "warn";
}

function impactItemTone(item) {
  if (item?.severity === "error") {
    return "bad";
  }
  if (item?.severity === "warning") {
    return "warn";
  }
  return "good";
}

function transactionTrustTone(trust) {
  const tones = {
    no_counterparty: "good",
    recognized: "good",
    mixed: "warn",
    unrecognized: "warn",
  };
  return tones[trust?.level] ?? "warn";
}

function trustItemTone(item) {
  if (item?.severity === "error") {
    return "bad";
  }
  if (item?.severity === "warning" || item?.status === "unknown") {
    return "warn";
  }
  return "good";
}

function transactionTrustItemTitle(item) {
  if (item?.label) {
    return item.protocol ? `${item.protocol} ${item.label}` : item.label;
  }
  const labels = {
    transaction_to: "Transaction recipient",
    approval_spender: "Approval spender",
    approval_operator: "Approval operator",
  };
  return labels[item?.role] ?? "Counterparty";
}

function transactionPolicyDecisionLabel(policy) {
  return String(policy?.decision ?? "blocked").replaceAll("_", " ");
}

function transactionSimulationLabel(simulation) {
  const mode = String(simulation?.mode ?? "simulation").replaceAll("_", " ");
  const status = String(simulation?.status ?? "unknown").replaceAll("_", " ");
  return `${mode} · ${status}`;
}

function policyBlockerTitle(code) {
  const labels = {
    live_simulation_required: "No live simulation",
    invalid_transaction_request: "Invalid transaction",
    simulation_provider_failed: "Simulation failed",
    unknown_calldata: "Unknown calldata",
    high_risk_unlimited_approval: "Unlimited token approval",
    high_risk_operator_approval: "Approval for all",
  };
  return labels[code] ?? String(code ?? "Policy reason").replaceAll("_", " ");
}

function simulationWarningTitle(code) {
  const labels = {
    native_value_transfer: "Native value transfer",
    unknown_function_selector: "Unknown function",
    unlimited_token_approval: "Unlimited token approval",
    operator_approval_for_all: "Approval for all",
  };
  return labels[code] ?? String(code ?? "Simulation warning").replaceAll("_", " ");
}

function policyBlockerCoversWarning(blockers, warning) {
  const warningCode = warning?.code;
  if (!warningCode) {
    return false;
  }
  const covered = {
    invalid_transaction_request: ["invalid_transaction_params", "invalid_transaction_field"],
    simulation_provider_failed: [
      "simulation_client_error",
      "simulation_provider_unavailable",
      "simulation_provider_response_unreadable",
      "simulation_provider_response_malformed",
      "simulation_provider_http_error",
      "simulation_provider_error",
      "simulation_provider_result_error",
    ],
    unknown_calldata: ["unknown_function_selector"],
    high_risk_unlimited_approval: ["unlimited_token_approval"],
    high_risk_operator_approval: ["operator_approval_for_all"],
  };
  return blockers.some((blocker) => (covered[blocker.code] ?? []).includes(warningCode));
}

function summaryGrid(rows) {
  const grid = document.createElement("dl");
  grid.className = "summary-grid";
  for (const [label, value] of rows) {
    const dt = document.createElement("dt");
    dt.textContent = label;
    const dd = document.createElement("dd");
    dd.textContent = valueOrDash(value);
    grid.append(dt, dd);
  }
  return grid;
}

function approveActionForRequest(request) {
  if (request.kind === "account_connection") {
    return { label: "Connect", decision: "approve", disabled: false };
  }
  if (request.kind === "network_switch") {
    const label = request.summary?.intent === "add_network" ? "Add Network" : "Switch Network";
    return { label, decision: "approve", disabled: false };
  }
  if (request.kind === "watch_asset") {
    return { label: "Add Token", decision: "approve", disabled: false };
  }
  if (request.kind === "personal_sign") {
    return { label: "Sign Message", decision: "approve", disabled: false };
  }
  if (request.kind === "transaction") {
    const policy = request.summary?.policy;
    const guidance = request.summary?.guidance;
    if (policy?.canSign) {
      return {
        label: userActionLabel(guidance?.primaryAction, "Approve & Send"),
        decision: "approve",
        disabled: false,
      };
    }
    if (policy?.overrideAllowed) {
      return {
        label: userActionLabel(guidance?.primaryAction, "Approve with Caution"),
        decision: "approve_with_risk",
        disabled: false,
        tone: "danger",
      };
    }
    return {
      label: userActionLabel(guidance?.primaryAction, "Cannot approve"),
      decision: "approve",
      disabled: true,
      disabledReason: guidance?.nextStep ?? guidance?.message ?? "Transaction policy blocks signing",
    };
  }
  if (request.kind === "typed_data") {
    if (typedDataSigningAllowed(request)) {
      return { label: "Approve Permission", decision: "approve", disabled: false };
    }
    return { label: "Blocked", decision: "approve", disabled: true };
  }
  return { label: "Approve Dry Run", decision: "approve", disabled: false };
}

function userActionLabel(value, fallback) {
  const labels = {
    "Approve Transaction": "Approve & Send",
    "Approve High Risk": "Approve with Caution",
    "Cannot Sign": "Cannot approve",
    "Ready for ordinary approval": "Ready to approve",
    "High-risk approval": "Caution required",
    "Ordinary approval": "Ready to approve",
  };
  return labels[value] ?? value ?? fallback;
}

function renderSimulationReport(simulation, policy, assetContext = null) {
  if (!simulation || typeof simulation !== "object") {
    return null;
  }

  const panel = document.createElement("div");
  panel.className = "simulation-report";

  const header = document.createElement("div");
  header.className = "simulation-header";
  header.append(
    simulationBadge(simulation.mode ?? "simulation"),
    simulationBadge(simulation.status ?? "unknown"),
  );

  if (simulation.decodedCall) {
    const protocol = decodedCallProtocolLabel(simulation.decodedCall);
    if (protocol !== "-") {
      header.append(simulationBadge(protocol));
    }
    header.append(simulationBadge(simulation.decodedCall.function ?? "decoded"));
  }
  panel.append(header);

  if (simulation.decodedCall?.arguments?.length) {
    const args = document.createElement("dl");
    args.className = "simulation-facts";
    for (const item of simulation.decodedCall.arguments) {
      const name = document.createElement("dt");
      name.textContent = item.name ?? "arg";
      const value = document.createElement("dd");
      value.textContent = item.value ?? "-";
      args.append(name, value);
    }
    panel.append(args);
  }

  if (simulation.approvals?.length) {
    panel.append(simulationList("Approvals", simulation.approvals, assetContext));
  }

  if (simulation.assetTransfers?.length) {
    panel.append(simulationList("Transfers", simulation.assetTransfers, assetContext));
  }

  if (simulation.nativeValue) {
    panel.append(simulationList("Native Value", [simulation.nativeValue]));
  }

  if (simulation.warnings?.length) {
    const warnings = document.createElement("ul");
    warnings.className = "simulation-warnings";
    for (const warning of simulation.warnings) {
      const item = document.createElement("li");
      item.dataset.severity = warning.severity ?? "warning";
      item.textContent = `${warning.code ?? "warning"}: ${warning.message ?? ""}`;
      warnings.append(item);
    }
    panel.append(warnings);
  }

  if (assetContext?.tokens?.length || assetContext?.errors?.length) {
    panel.append(renderAssetContext(assetContext));
  }

  if (policy && typeof policy === "object") {
    panel.append(renderPolicyGate(policy));
  }

  return panel;
}

function renderPolicyGate(policy) {
  const gate = document.createElement("div");
  gate.className = "policy-gate";

  const title = document.createElement("div");
  title.className = "policy-title";
  title.append(
    simulationBadge(`policy ${policy.decision ?? "blocked"}`),
    simulationBadge(`can sign ${policy.canSign ? "yes" : "no"}`),
    simulationBadge(`can broadcast ${policy.canBroadcast ? "yes" : "no"}`),
    simulationBadge(`override ${policy.overrideAllowed ? "yes" : "no"}`),
  );
  gate.append(title);

  if (policy.blockers?.length) {
    const blockers = document.createElement("ul");
    blockers.className = "policy-blockers";
    for (const blocker of policy.blockers) {
      const item = document.createElement("li");
      const override = blocker.overrideable ? "overrideable" : "blocking";
      item.textContent = `${blocker.code ?? "blocked"} (${override}): ${blocker.message ?? ""}`;
      blockers.append(item);
    }
    gate.append(blockers);
  }

  return gate;
}

function simulationBadge(value) {
  const badge = document.createElement("span");
  badge.className = "simulation-badge";
  badge.textContent = String(value).replaceAll("_", " ");
  return badge;
}

function renderAssetContext(assetContext) {
  const details = document.createElement("details");
  details.className = "simulation-details";
  details.open = false;
  const summary = document.createElement("summary");
  summary.textContent = `Assets · ${String(assetContext.status ?? "metadata").replaceAll("_", " ")}`;
  const body = document.createElement("ul");
  body.className = "compact-list";
  for (const token of assetContext.tokens ?? []) {
    const item = document.createElement("li");
    const metadata = token.metadata ?? {};
    const symbol = metadata.symbol ?? token.assetKind ?? "token";
    const name = metadata.name ? ` · ${metadata.name}` : "";
    const error = token.metadataError ? " · metadata unavailable" : "";
    item.textContent = `${symbol}${name} · ${shortAddress(token.contractAddress)}${error}`;
    body.append(item);
  }
  for (const error of assetContext.errors ?? []) {
    const item = document.createElement("li");
    item.textContent = `${error.scope ?? "metadata"}: ${error.message ?? "unavailable"}`;
    body.append(item);
  }
  details.append(summary, body);
  return details;
}

function simulationList(label, values, assetContext = null) {
  const details = document.createElement("details");
  details.className = "simulation-details";
  details.open = true;
  const summary = document.createElement("summary");
  summary.textContent = label;
  const body = document.createElement("ul");
  body.className = "compact-list";
  for (const value of values) {
    const item = document.createElement("li");
    item.textContent = formatSimulationItem(value, assetContext);
    body.append(item);
  }
  details.append(summary, body);
  return details;
}

function formatSimulationItem(value, assetContext = null) {
  if (!value || typeof value !== "object") {
    return valueOrDash(value);
  }
  if (value.assetKind && (value.spender || value.operator)) {
    const actor = value.spender ?? value.operator;
    const token = tokenContextForContract(assetContext, value.contract);
    const amount = value.amount ? formatReviewTokenAmount(value.amount, token) : value.approved;
    return `${assetDisplayName(value, token)} approval to ${shortAddress(actor)} · ${valueOrDash(amount)}`;
  }
  if (value.assetKind && (value.from || value.to)) {
    const token = tokenContextForContract(assetContext, value.contract);
    const amount = value.amount
      ? formatReviewTokenAmount(value.amount, token)
      : value.tokenId?.decimal
        ? `token ${value.tokenId.decimal}`
        : "-";
    return `${assetDisplayName(value, token)} ${shortAddress(value.from)} -> ${shortAddress(value.to)} · ${amount}`;
  }
  if (value.hex && value.decimal) {
    return formatNativeBalance(value.hex);
  }
  return JSON.stringify(value);
}

function formatTokenAmount(amount) {
  return amount?.decimal ?? amount?.hex ?? "-";
}

function formatReviewTokenAmount(amount, token) {
  const metadata = token?.metadata ?? {};
  const symbol = metadata.symbol ?? token?.assetKind ?? "";
  if (isMaxUint256(amount?.hex)) {
    return `Unlimited ${symbol}`.trim();
  }
  if (Number.isInteger(metadata.decimals) && amount?.hex) {
    return formatTokenBalance(amount.hex, metadata.decimals, symbol);
  }
  const raw = formatTokenAmount(amount);
  return `${raw}${symbol ? ` ${symbol}` : ""}`.trim();
}

function tokenContextForContract(assetContext, contract) {
  if (!assetContext?.tokens?.length || typeof contract !== "string") {
    return null;
  }
  return (
    assetContext.tokens.find(
      (token) =>
        typeof token.contractAddress === "string" &&
        token.contractAddress.toLowerCase() === contract.toLowerCase(),
    ) ?? null
  );
}

function assetDisplayName(value, token) {
  const metadata = token?.metadata ?? {};
  return metadata.symbol ?? metadata.name ?? value.assetKind ?? "asset";
}

function transactionAssetContextLabel(assetContext) {
  if (!assetContext?.tokens?.length) {
    return "-";
  }
  const labels = assetContext.tokens
    .slice(0, 3)
    .map((token) => token.metadata?.symbol ?? shortAddress(token.contractAddress))
    .filter(Boolean);
  const suffix = assetContext.tokens.length > labels.length ? ` +${assetContext.tokens.length - labels.length}` : "";
  return `${labels.join(", ")}${suffix}`;
}

function transactionProtocolLabel(simulation) {
  return decodedCallProtocolLabel(simulation?.decodedCall);
}

function decodedCallProtocolLabel(call) {
  const labels = {
    uniswap_v2_router: "Uniswap V2",
    uniswap_v3_swap_router: "Uniswap V3",
    uniswap_universal_router: "Uniswap Universal Router",
    aave_v3_pool: "Aave V3",
    multicall: "Multicall",
  };
  const standard = call?.standard;
  if (!standard || standard === "unknown") {
    return "-";
  }
  return labels[standard] ?? "-";
}

function typedPermitIntentLabel(intent) {
  const labels = {
    erc20_permit: "ERC-20 Permit",
    permit2_permit_single: "Permit2 Single",
    permit2_permit_batch: "Permit2 Batch",
    permit2_transfer_from: "Permit2 Transfer",
    permit2_batch_transfer_from: "Permit2 Batch Transfer",
  };
  return labels[intent] ?? valueOrDash(intent);
}

function typedDataDomainLabel(domain) {
  if (!domain || typeof domain !== "object") {
    return "-";
  }
  const name = domain.name ?? "Typed Data";
  const contract = domain.verifyingContract ? ` · ${shortAddress(domain.verifyingContract)}` : "";
  const chain = domain.chainId != null ? ` · chain ${domain.chainId}` : "";
  return `${name}${contract}${chain}`;
}

function typedPermitTokenLabel(permit) {
  if (Array.isArray(permit.tokens) && permit.tokens.length > 0) {
    const first = permit.tokens
      .slice(0, 3)
      .map((item) => shortAddress(item.token))
      .join(", ");
    const suffix = permit.tokenCount > 3 ? ` +${permit.tokenCount - 3}` : "";
    return `${first}${suffix}`;
  }
  return shortAddress(permit.token);
}

function typedPermitAmountLabel(permit) {
  if (permit.amount != null) {
    return valueOrDash(permit.amount);
  }
  if (Array.isArray(permit.tokens) && permit.tokens.length > 0) {
    const amounts = permit.tokens
      .slice(0, 3)
      .map((item) => valueOrDash(item.amount))
      .join(", ");
    const suffix = permit.tokenCount > 3 ? ` +${permit.tokenCount - 3}` : "";
    return `${amounts}${suffix}`;
  }
  return "-";
}

function typedPermitDeadlineLabel(permit) {
  return valueOrDash(permit.deadline ?? permit.expiration);
}

function typedDataSigningAllowed(request) {
  return (
    request?.method === "eth_signTypedData_v4" &&
    [
      "erc20_permit",
      "permit2_permit_single",
      "permit2_permit_batch",
      "permit2_transfer_from",
      "permit2_batch_transfer_from",
    ].includes(request?.summary?.typedData?.intent)
  );
}

function isMaxUint256(hex) {
  return (
    typeof hex === "string" &&
    hex.toLowerCase() === `0x${"f".repeat(64)}`
  );
}

function formatReviewReason(request) {
  const status = request.status ?? "pending";
  if (status === "approved") {
    if (request.kind === "account_connection") {
      return "Connection approved; the app can now see this account.";
    }
    if (request.kind === "network_switch") {
      const action = request.summary?.intent === "add_network" ? "Network add" : "Network switch";
      return `${action} approved; applying trusted network settings.`;
    }
    if (request.kind === "watch_asset") {
      return "Token add approved; it will appear in the trusted Assets view.";
    }
    if (request.kind === "personal_sign") {
      return "Approved locally; waiting for the signing service.";
    }
    if (request.kind === "typed_data") {
      return "Permit approved locally; waiting for the signing service.";
    }
    if (request.kind === "transaction") {
      const highRisk = request.decision?.decision === "approve_with_risk";
      return `${highRisk ? "High-risk transaction approved" : "Transaction approved"}; waiting for signing and broadcast.`;
    }
    return "Approved locally.";
  }
  if (status === "completed") {
    if (request.kind === "account_connection") {
      return `Connected account ${shortAddress(request.execution?.address)}.`;
    }
    if (request.kind === "network_switch") {
      if (request.summary?.intent === "add_network") {
        return "Network add completed; active session network was not changed.";
      }
      return "Network switched for this app session.";
    }
    if (request.kind === "watch_asset") {
      return "Token added to the trusted Portfolio view for this app session.";
    }
    return "Request completed.";
  }
  if (status === "signed") {
    const address = request.execution?.address ?? "unknown address";
    const messageHash = request.execution?.messageHash ?? "unknown hash";
    if (request.kind === "transaction") {
      return `Broadcast by ${address}; transaction hash ${messageHash}`;
    }
    return `Signed by ${address}; message hash ${messageHash}`;
  }
  if (status === "sign_failed") {
    if (request.kind === "network_switch") {
      const action = request.summary?.intent === "add_network" ? "Network add" : "Network switch";
      return `${action} failed: ${request.execution?.error ?? "unknown error"}`;
    }
    if (request.kind === "watch_asset") {
      return `Token add failed: ${request.execution?.error ?? "unknown error"}`;
    }
    return `Signing failed: ${request.execution?.error ?? "unknown error"}`;
  }
  if (status === "rejected") {
    return "Rejected locally; the app cannot continue this request.";
  }
  if (status === "expired") {
    return "Expired before approval.";
  }
  return request.blockedReason ?? "Waiting for your decision.";
}

function valueOrDash(value) {
  if (value === null || value === undefined || value === "") {
    return "-";
  }
  return String(value);
}

function shortAddress(value) {
  if (typeof value !== "string" || value.length < 14) {
    return valueOrDash(value);
  }
  return `${value.slice(0, 8)}...${value.slice(-6)}`;
}

function shortHash(value) {
  if (typeof value !== "string" || value.length < 18) {
    return valueOrDash(value);
  }
  return `${value.slice(0, 10)}...${value.slice(-8)}`;
}

function shortOrigin(value) {
  if (typeof value !== "string") {
    return valueOrDash(value);
  }
  try {
    const url = new URL(value);
    return url.host || value;
  } catch {
    return value;
  }
}

function formatTransactionValue(summary) {
  const nativeValue = summary?.simulation?.nativeValue;
  if (nativeValue?.hex) {
    return formatNativeBalance(nativeValue.hex);
  }
  if (typeof summary?.value === "string") {
    return formatNativeBalance(summary.value);
  }
  return "0 ETH";
}

function transactionCallLabel(simulation) {
  const call = simulation?.decodedCall;
  if (call?.function) {
    return call.function;
  }
  const selector = simulation?.transaction?.selector;
  if (selector) {
    return `unknown ${selector}`;
  }
  return "native transfer";
}

function formatNativeBalance(hexQuantity) {
  try {
    const wei = BigInt(hexQuantity);
    const whole = wei / 1_000_000_000_000_000_000n;
    const fraction = wei % 1_000_000_000_000_000_000n;
    if (fraction === 0n) {
      return `${whole.toString()} ETH`;
    }
    const fractionText = fraction.toString().padStart(18, "0").slice(0, 6).replace(/0+$/, "");
    return `${whole.toString()}.${fractionText || "0"} ETH`;
  } catch {
    return valueOrDash(hexQuantity);
  }
}

function formatTokenBalance(hexQuantity, decimals, symbol) {
  try {
    const value = BigInt(hexQuantity);
    const decimalPlaces = Number.isInteger(decimals) && decimals >= 0 ? Math.min(decimals, 255) : 0;
    const digits = value.toString(10);
    if (decimalPlaces === 0) {
      return `${digits} ${symbol ?? ""}`.trim();
    }
    const padded = digits.padStart(decimalPlaces + 1, "0");
    const whole = padded.slice(0, -decimalPlaces) || "0";
    const fraction = padded
      .slice(-decimalPlaces)
      .slice(0, 6)
      .replace(/0+$/, "");
    return `${whole}${fraction ? `.${fraction}` : ""} ${symbol ?? ""}`.trim();
  } catch {
    return `${valueOrDash(hexQuantity)} ${symbol ?? ""}`.trim();
  }
}

function formatHexInteger(hexQuantity) {
  try {
    return BigInt(hexQuantity).toString(10);
  } catch {
    return valueOrDash(hexQuantity);
  }
}

function formatTime(unixMs) {
  if (!Number.isFinite(unixMs)) {
    return "-";
  }
  return new Date(unixMs).toLocaleTimeString();
}

function renderAccount(account) {
  latestAccount = account;
  accountAddress.textContent = account.address ?? "Unavailable";
  chainId.textContent = account.chainId ?? "-";
  if (latestStatus?.network) {
    networkName.textContent = formatNetwork(latestStatus.network);
  }
  signerHelper.textContent = formatSignerHelper(account.signerHelper);
  if (account.metadata) {
    output.textContent = JSON.stringify(account, null, 2);
  }
  renderSessionReadiness();
}

function renderCapabilities(value) {
  capabilities.replaceChildren();
  const entries = Object.entries(value);
  if (entries.length === 0) {
    const item = document.createElement("li");
    item.textContent = "No capabilities reported";
    capabilities.append(item);
    return;
  }

  for (const [name, enabled] of entries) {
    const item = document.createElement("li");
    const marker = document.createElement("span");
    marker.className = enabled === false ? "capability-dot muted" : "capability-dot good";
    item.append(marker, document.createTextNode(`${labelize(name)}: ${formatCapabilityValue(enabled)}`));
    capabilities.append(item);
  }
}

function formatCapabilityValue(value) {
  if (value === true) {
    return "enabled";
  }
  if (value === false) {
    return "disabled";
  }
  return String(value);
}

function formatDevice(value) {
  if (!value) {
    return "-";
  }
  if (value.kind === "gbx_cart") {
    return `${value.kind} ${value.saveType ?? ""} ${value.port ?? "auto port"}`.trim();
  }
  if (value.kind === "file") {
    return `${value.kind} ${value.path ?? ""}`.trim();
  }
  return JSON.stringify(value);
}

function formatWallet(value) {
  if (!value) {
    return "-";
  }
  const suffix = value.mock ? "mock" : "real";
  return `${value.kind ?? "wallet"} ${suffix}`.trim();
}

function formatRpc(value) {
  if (!value) {
    return "not configured";
  }
  const network = value.network ? ` ${value.network}` : "";
  return `${value.kind ?? "rpc"}${network} ${value.timeoutMs ?? "-"}ms`.trim();
}

function formatRpcWithHealth(rpc, health) {
  const base = formatRpc(rpc);
  if (!health) {
    return base;
  }
  const status = rpcHealthReadinessText(health);
  return `${base} · ${status}`;
}

function rpcHealthTone(health) {
  if (health?.healthy) {
    return "good";
  }
  if (["missing", "wrong_chain"].includes(health?.status)) {
    return "warn";
  }
  return "bad";
}

function rpcHealthSummaryText(health) {
  if (health?.healthy) {
    return "RPC healthy";
  }
  if (health?.status === "missing") {
    return "RPC not configured";
  }
  if (health?.status === "wrong_chain") {
    return "Wrong chain";
  }
  if (health?.status === "invalid_chain") {
    return "Invalid chain response";
  }
  return "RPC unavailable";
}

function rpcHealthReadinessText(health) {
  if (health?.healthy) {
    return "Healthy";
  }
  if (health?.status === "missing") {
    return "Missing";
  }
  if (health?.status === "wrong_chain") {
    return "Wrong chain";
  }
  return "Error";
}

function rpcHealthChainText(health) {
  const observed = health?.observedChainId;
  const expected = health?.expectedChainId;
  if (observed && health?.chainMatches) {
    return `${observed} matches`;
  }
  if (observed && expected) {
    return `${observed} expected ${expected}`;
  }
  if (expected) {
    return `expected ${expected}`;
  }
  return "-";
}

function dappSessionOneLine(session) {
  if (!session) {
    return null;
  }
  if (!session.open) {
    return "No app open";
  }
  const origin = session.origin ? shortOrigin(session.origin) : null;
  const target = session.targetLabel ?? "dApp";
  const status = dappLoadStatusText(session);
  return origin ? `${target} · ${origin} · ${status}` : `${target} · ${status}`;
}

function dappLoadStatusText(session) {
  if (!session?.open) {
    return "Not opened";
  }
  const status = String(session?.loadStatus ?? "not_loaded").replaceAll("_", " ");
  return labelize(status);
}

function dappLoadTone(session) {
  if (!session?.open || session?.loadStatus === "not_loaded") {
    return "warn";
  }
  if (session?.loadStatus === "loaded") {
    return "good";
  }
  if (["loading", "opening", "navigating", "navigation_requested"].includes(session?.loadStatus)) {
    return "busy";
  }
  return "warn";
}

function rpcHealthDetailText(health) {
  if (health?.error?.message) {
    return `${health.error.scope ?? "rpc"}: ${health.error.message}`;
  }
  if (health?.healthy) {
    return "Alchemy RPC is reachable; token and endpoint are hidden";
  }
  return "Token and endpoint are hidden";
}

function formatNetwork(value) {
  if (!value) {
    return "-";
  }
  const label = value.name ?? value.chainId ?? "network";
  const rpc = value.alchemyNetwork ? ` ${value.alchemyNetwork}` : "";
  return `${label}${rpc}`.trim();
}

function nativeSymbolForStatus(status) {
  const active = status?.chainId;
  const chain = (status?.supportedChains ?? []).find((item) => sameChainId(item.chainId, active));
  return chain?.nativeSymbol ?? "ETH";
}

function networkOptionLabel(value) {
  const label = value.name ?? value.chainId ?? "Network";
  const chain = value.chainId ? ` (${value.chainId})` : "";
  return `${label}${chain}`;
}

function sameChainId(left, right) {
  return String(left ?? "").toLowerCase() === String(right ?? "").toLowerCase();
}

function formatSignerHelper(value) {
  if (!value) {
    return "-";
  }
  const readiness = value.readiness
    ? labelize(String(value.readiness).replaceAll("_", " "))
    : value.exists === false
      ? "Missing"
      : "Ready";
  const location = value.location ? ` · ${labelize(String(value.location).replaceAll("_", " "))}` : "";
  const sandbox = value.sandbox ? ` · ${value.sandbox}` : "";
  const pin =
    value.hashPinned === true
      ? value.hashMatches === false
        ? " · hash mismatch"
        : " · hash pinned"
      : "";
  const hash = value.blake3 ? ` · ${String(value.blake3).slice(0, 12)}` : "";
  return `${readiness}${location}${sandbox}${pin}${hash}`;
}

function formatKeychainHelperAccessResult(value) {
  if (!value) {
    return "Signing access unknown";
  }
  const status = value.status ? labelize(String(value.status)) : "Authorized";
  const cdhash = value.helperCdhash ? ` · cdhash ${String(value.helperCdhash).slice(0, 12)}` : "";
  const signingService = value.helper ? ` · ${formatSignerHelper(value.helper)}` : "";
  return `Signing access ${status}${cdhash}${signingService}`;
}

function labelize(value) {
  return value
    .replace(/([A-Z])/g, " $1")
    .replace(/^./, (char) => char.toUpperCase());
}

function textSpan(text) {
  const span = document.createElement("span");
  span.textContent = text;
  return span;
}

function parsePathList(value) {
  return value
    .split(/[\n,]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function setBusy(action) {
  setBridgeState(action, "busy");
}

function setBridgeState(text, tone) {
  bridgeState.textContent = text;
  bridgeState.dataset.tone = tone;
}

function renderError(error) {
  setBridgeState("Error", "bad");
  output.textContent = JSON.stringify(
    {
      error: {
        message: error?.message ?? String(error),
      },
    },
    null,
    2,
  );
}

refreshStatusButton.addEventListener("click", () => {
  refreshStatus().catch(() => {});
});
refreshRpcHealthButton.addEventListener("click", () => {
  refreshRpcHealth().catch(() => {});
});
networkSelect.addEventListener("change", () => {
  updateNetworkSwitchState();
});
switchNetworkButton.addEventListener("click", () => {
  switchNetwork().catch(() => {});
});
walletActionConnectButton.addEventListener("click", () => {
  toggleWalletConnection().catch(() => {});
});
walletActionSendButton.addEventListener("click", () => {
  nativeSendTo?.focus();
});
walletActionDefiButton.addEventListener("click", () => {
  setActiveWorkspace("defi");
});
defiReviewCallout?.addEventListener("click", () => {
  focusReviewPanel();
});
defiPrimaryApprovalApprove?.addEventListener("click", () => {
  if (!defiPrimaryApprovalRequest) {
    return;
  }
  const action = approveActionForRequest(defiPrimaryApprovalRequest);
  if (action.disabled) {
    focusReviewPanel();
    return;
  }
  decideReviewRequest(defiPrimaryApprovalRequest, action.decision).catch(() => {});
});
defiPrimaryApprovalReject?.addEventListener("click", () => {
  if (!defiPrimaryApprovalRequest) {
    return;
  }
  decideReviewRequest(defiPrimaryApprovalRequest, "reject").catch(() => {});
});
defiPrimaryApprovalDetails?.addEventListener("click", () => {
  focusReviewPanel();
});
for (const tab of workspaceTabs) {
  tab.addEventListener("click", () => {
    setActiveWorkspace(tab.dataset.workspaceTab);
  });
}
refreshPortfolioButton.addEventListener("click", () => {
  refreshPortfolio().catch(() => {});
});
nativeSendForm.addEventListener("submit", (event) => {
  sendNativeTransfer(event).catch(() => {});
});
tokenSendForm.addEventListener("submit", (event) => {
  sendTokenTransfer(event).catch(() => {});
});
refreshActivityButton.addEventListener("click", () => {
  refreshTransactionActivity(true, false).catch(() => {});
});
refreshReceiptsButton.addEventListener("click", () => {
  refreshTransactionActivity(true, true).catch(() => {});
});
connectCardButton.addEventListener("click", () => {
  connectCard().catch(() => {});
});
authorizeKeychainHelperButton.addEventListener("click", () => {
  authorizeKeychainHelper().catch(() => {});
});
openDappButton.addEventListener("click", () => {
  openDapp("local", "Local Test").catch(() => {});
});
openCustomDappButton.addEventListener("click", () => {
  openDapp(dappUrl.value).catch(() => {});
});
openUniswapButton.addEventListener("click", () => {
  dappUrl.value = "https://app.uniswap.org/";
  openDapp("uniswap", "Uniswap").catch(() => {});
});
openAaveButton.addEventListener("click", () => {
  dappUrl.value = "https://app.aave.com/";
  openDapp("aave", "Aave").catch(() => {});
});
defiActionUniswapButton.addEventListener("click", () => {
  dappUrl.value = "https://app.uniswap.org/";
  openDapp("uniswap", "Uniswap").catch(() => {});
});
defiActionAaveButton.addEventListener("click", () => {
  dappUrl.value = "https://app.aave.com/";
  openDapp("aave", "Aave").catch(() => {});
});
defiActionConnectButton.addEventListener("click", () => {
  dappUrl.focus();
  dappUrl.select();
  document.querySelector(".defi-browser-panel")?.scrollIntoView({ behavior: "smooth", block: "start" });
});
openLocalDappButton.addEventListener("click", () => {
  openDapp("local", "Local Test").catch(() => {});
});
dappNavBackButton.addEventListener("click", () => {
  navigateDapp("back").catch(() => {});
});
dappNavForwardButton.addEventListener("click", () => {
  navigateDapp("forward").catch(() => {});
});
dappNavReloadButton.addEventListener("click", () => {
  navigateDapp("reload").catch(() => {});
});
dappNavHomeButton.addEventListener("click", () => {
  navigateDapp("home").catch(() => {});
});
refreshConnectionsButton.addEventListener("click", () => {
  refreshConnectedSites().catch(() => {});
});
refreshProviderEventsButton.addEventListener("click", () => {
  refreshProviderEvents().catch(() => {});
});
clearProviderEventsButton.addEventListener("click", () => {
  clearProviderEvents().catch(() => {});
});
createVaultButton.addEventListener("click", () => {
  createVault().catch(() => {});
});
confirmOverwrite.addEventListener("change", () => {
  updateCreateVaultActionState();
});
vaultGeneration.addEventListener("input", () => {
  resetCreateCompletion();
});
recoveryOutDir.addEventListener("input", () => {
  resetCreateCompletion();
});
chooseRecoveryOutDirButton.addEventListener("click", () => {
  chooseRecoveryOutDir().catch(() => {});
});
for (const card of restoreSchemeCards) {
  card.addEventListener("click", () => {
    setRecoveryScheme(card.dataset.recoveryScheme);
  });
}
restoreFileSlots?.addEventListener("click", (event) => {
  const button = event.target.closest("[data-recovery-slot]");
  if (!button || !restoreFileSlots.contains(button)) {
    return;
  }
  chooseRecoveryFileForSlot(button.dataset.recoverySlot).catch(() => {});
});
clearRecoveryFilesButton.addEventListener("click", () => {
  clearRecoveryFileBuckets();
  renderRecoveryInputStatus();
});
recoveryFilePaths.addEventListener("input", () => {
  clearRecoveryFileBuckets({ syncInput: false, resetOutcomes: false });
  vaultBackupPath.value = recoveryVaultSourcePath();
  markRecoverySelectionChanged();
  renderRecoveryInputStatus();
});
vaultBackupPath?.addEventListener("input", () => {
  latestRecoveryRecoverOutcome = null;
  renderRecoveryInputStatus();
});
recoverOverwrite.addEventListener("change", () => {
  renderRecoveryInputStatus();
});
recoverVaultButton.addEventListener("click", () => {
  recoverVault().catch(() => {});
});
clearRecoveryPlanButton.addEventListener("click", () => {
  clearRecoveryPlan().catch(() => {});
});
refreshReviewButton.addEventListener("click", () => {
  refreshReviewQueue(true).catch(() => {});
});
clearReviewButton.addEventListener("click", () => {
  clearReviewQueue().catch(() => {});
});

renderPortfolioBaseline();
renderRpcHealthBaseline();
renderTransactionActivityBaseline();
renderRecoveryPanel();
updateCreateVaultActionState();
renderRecoveryInputStatus();
setActiveWorkspace(activeWorkspace, { persist: false });
updateWorkspaceReviewCounts();
renderSessionReadiness();
renderCompatibilityStatus();
refreshRecoveryState(false).catch(() => {});
refreshDappSession(false).catch(() => {});
refreshStatus().catch(() => {});
setInterval(() => {
  refreshReviewQueue(false).catch(() => {});
  refreshProviderEvents(false).catch(() => {});
  refreshTransactionActivity(false, false).catch(() => {});
  refreshDappSession(false).catch(() => {});
}, 2500);
