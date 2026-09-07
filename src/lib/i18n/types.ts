export type UiLanguage = 'system' | 'en' | 'ru';
export type ResolvedLanguage = 'en' | 'ru';

export interface TranslationDictionary {
  // Navigation
  nav: {
    history: string;
    general: string;
    dictation: string;
    providers: string;
  };
  // TopBar / Global
  topbar: {
    historySubtitle: string;
    dictationTitle: string;
    searchPlaceholder: string;
    record: string;
    stop: string;
  };
  // Modes
  modes: {
    raw: string;
    minimal: string;
    balanced: string;
    business: string;
  };
  // History View & List
  history: {
    today: string;
    yesterday: string;
    earlier: string;
    tabResult: string;
    tabRaw: string;
    tabDiff: string;
    itemsCount: (count: number) => string;
    emptyTitle: string;
    emptySubtitle: string;
    searchEmptyTitle: string;
    searchEmptySubtitle: (query: string) => string;
    historyDisabledTitle: string;
    historyDisabledSubtitle: string;
    enableHistory: string;
    copyTooltip: string;
    deleteTooltip: string;
    copied: string;
    emptyRecordTitle: string;
    itemDeleted: string;
    historyCleared: string;
    itemRestored: string;
    historyRestored: string;
    failedToPersist: string;
    diffAdded: string;
    diffRemoved: string;
  };
  // Settings Panel
  settings: {
    sectionApp: string;
    sectionAppearance: string;
    sectionHistory: string;
    sectionMicrophone: string;
    sectionSpeechRecognition: string;
    sectionTextProcessing: string;
    pressKey: string;
    signalLevel: string;
    providersHeading: string;
    keySaved: string;
    customRgbPicker: string;
    colorOrange: string;
    colorBlue: string;
    colorEmerald: string;
    colorPurple: string;
    colorPink: string;
    colorMonochrome: string;
    clearHistoryModalTitle: string;
    clearHistoryModalDesc: string;
    microphone: string;
    microphoneDefault: string;
    testMic: string;
    hotkey: string;
    changeHotkey: string;
    cancelHotkey: string;
    sttProvider: string;
    sttModel: string;
    llmProvider: string;
    llmModel: string;
    postprocessing: string;
    processingMode: string;
    transcriptionLang: string;
    interfaceLang: string;
    saveHistory: string;
    startWithWindows: string;
    theme: string;
    accentColor: string;
    themeSystem: string;
    themeLight: string;
    themeDark: string;
    providersTitle: string;
    notConfigured: string;
    testProvider: string;
    testingProvider: string;
    providerConnected: string;
    providerError: string;
    enterApiKey: (name: string) => string;
    showKey: string;
    hideKey: string;
    saveKey: string;
    clearKey: string;
    historyCountLabel: string;
    clearHistoryBtn: string;
    collapseSidebar: string;
    expandSidebar: string;
    colorHue: string;
    colorTone: string;
    eyedropperTooltip: string;
    eyedropperPickingTooltip: string;
    unsupportedHotkey: (key: string) => string;
    testMicFailed: string;
  };
  // Languages
  languages: {
    auto: string;
    ru: string;
    en: string;
    de: string;
    fr: string;
    es: string;
    system: string;
  };
  // HUD
  hud: {
    ready: string;
    listening: string;
    finishing: string;
    transcribing: string;
    refining: string;
    inserted: string;
    copied: string;
    cancelled: string;
    stop: string;
    retry: string;
    settings: string;
    error: string;
    noSpeechShort: string;
    pasteErrorShort: string;
    micErrorShort: string;
    networkErrorShort: string;
    invalidKeyShort: string;
  };
  // Errors
  errors: {
    permissionDenied: string;
    micNotFound: string;
    micBusy: string;
    invalidApiKey: string;
    networkError: string;
    providerUnavailable: string;
    recordingTooLong: string;
    noSpeech: string;
    noSpeechHint: string;
    pasteError: string;
    recognizedButNotInserted: string;
    recordingNotStarted: string;
    desktopOnly: string;
    defaultError: string;
  };
  // Common / Errors
  common: {
    copy: string;
    confirm: string;
    cancel: string;
    undo: string;
    selectPlaceholder: string;
    selectAriaLabel: string;
    closeToTray: string;
    defaultBadge: string;
    switchToDarkTheme: string;
    switchToLightTheme: string;
    textCopiedToast: string;
    copyFailedToast: string;
    failedToSaveSettings: string;
    failedToSaveTheme: string;
    failedToSaveMode: string;
    modeChanged: (name: string) => string;
  };
}
