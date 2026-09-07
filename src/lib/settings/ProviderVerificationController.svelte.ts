import { invoke } from '@tauri-apps/api/core';
import {
  createDefaultProviderApiKeys,
  isProviderId,
  PROVIDER_IDS,
  type ProviderApiKeys,
  type ProviderId,
} from './model.js';

type ProviderCheckState =
  | 'idle'
  | 'checking'
  | 'valid'
  | 'invalid_key'
  | 'error'
  | 'unverified';

interface ProviderStatusInfo {
  state: ProviderCheckState;
}

async function checkProviderKey(
  provider: string,
  key: string,
): Promise<ProviderStatusInfo> {
  if (!key || key.trim().length < 4) {
    return { state: 'invalid_key' };
  }

  const tauriInternals =
    typeof window !== 'undefined' ? window.__TAURI_INTERNALS__ : undefined;
  if (!tauriInternals?.invoke) {
    return { state: 'unverified' };
  }

  try {
    const ok = await invoke<boolean>('check_provider_api_key', {
      provider,
      apiKey: key.trim(),
    });
    return { state: ok ? 'valid' : 'invalid_key' };
  } catch {
    return { state: 'error' };
  }
}

interface ProviderVerificationOptions {
  persist: () => Promise<void>;
  deleteKey: (provider: ProviderId) => Promise<void>;
}

function createStatuses(
  keys: ProviderApiKeys,
  verifiedProviders: ProviderId[],
): Record<ProviderId, ProviderStatusInfo> {
  return Object.fromEntries(
    PROVIDER_IDS.map((provider) => [
      provider,
      {
        state: verifiedProviders.includes(provider)
          ? 'valid'
          : keys[provider].trim()
            ? 'unverified'
            : 'idle',
      },
    ]),
  ) as Record<ProviderId, ProviderStatusInfo>;
}

export class ProviderVerificationController {
  keys = $state<ProviderApiKeys>(createDefaultProviderApiKeys());
  verifiedProviders = $state<ProviderId[]>([]);
  statuses = $state<Record<ProviderId, ProviderStatusInfo>>(
    createStatuses(this.keys, this.verifiedProviders),
  );

  readonly #persist: () => Promise<void>;
  readonly #deleteKey: (provider: ProviderId) => Promise<void>;

  constructor({ persist, deleteKey }: ProviderVerificationOptions) {
    this.#persist = persist;
    this.#deleteKey = deleteKey;
  }

  hydrate(keys: ProviderApiKeys, verifiedProviders: ProviderId[]): void {
    const normalizedVerified = verifiedProviders.filter(
      (provider) => keys[provider].trim(),
    );
    const keysChanged = PROVIDER_IDS.some(
      (provider) => this.keys[provider] !== keys[provider],
    );
    const verificationChanged =
      normalizedVerified.length !== this.verifiedProviders.length ||
      normalizedVerified.some(
        (provider) => !this.verifiedProviders.includes(provider),
      );
    if (!keysChanged && !verificationChanged) return;

    this.keys = { ...keys };
    this.verifiedProviders = [...normalizedVerified];
    this.statuses = createStatuses(this.keys, this.verifiedProviders);
  }

  async verify(providerValue: string): Promise<void> {
    if (!isProviderId(providerValue)) return;
    const provider = providerValue;
    const key = this.keys[provider].trim();
    if (!key) return;

    const previousVerified = [...this.verifiedProviders];
    this.statuses[provider] = { state: 'checking' };
    try {
      const result = await checkProviderKey(provider, key);
      if (result.state === 'valid' || result.state === 'invalid_key') {
        this.setVerified(provider, result.state === 'valid');
        await this.#persist();
      }
      this.statuses[provider] = result;
    } catch {
      this.verifiedProviders = previousVerified;
      this.statuses[provider] = { state: 'error' };
    }
  }

  async save(providerValue: string, value: string): Promise<void> {
    if (!isProviderId(providerValue)) return;
    const provider = providerValue;
    const previousKeys = { ...this.keys };
    const previousStatus = this.statuses[provider];
    const previousVerified = [...this.verifiedProviders];

    this.keys = { ...this.keys, [provider]: value.trim() };
    this.setVerified(provider, false);
    this.statuses[provider] = {
      state: this.keys[provider] ? 'unverified' : 'idle',
    };
    try {
      await this.#persist();
      await this.verify(provider);
    } catch (error) {
      this.keys = previousKeys;
      this.verifiedProviders = previousVerified;
      this.statuses[provider] = previousStatus;
      throw error;
    }
  }

  async clear(providerValue: string): Promise<void> {
    if (!isProviderId(providerValue)) return;
    const provider = providerValue;
    const previousKeys = { ...this.keys };
    const previousStatus = this.statuses[provider];
    const previousVerified = [...this.verifiedProviders];

    this.keys = { ...this.keys, [provider]: '' };
    this.setVerified(provider, false);
    this.statuses[provider] = { state: 'idle' };
    try {
      await this.#deleteKey(provider);
      await this.#persist();
    } catch (error) {
      this.keys = previousKeys;
      this.verifiedProviders = previousVerified;
      this.statuses[provider] = {
        ...previousStatus,
        state: 'error',
      };
      throw error;
    }
  }

  private setVerified(provider: ProviderId, verified: boolean): void {
    this.verifiedProviders = verified
      ? [...new Set([...this.verifiedProviders, provider])]
      : this.verifiedProviders.filter((current) => current !== provider);
  }
}
