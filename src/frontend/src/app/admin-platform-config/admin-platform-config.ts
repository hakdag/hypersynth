import { KeyValuePipe } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';

import {
  AdminConfigApiService,
  PlatformConfig,
  UpdatePlatformConfigRequest,
} from '../admin-config-api.service';

type LoadState = 'loading' | 'ok' | 'error';

const PROVIDER_OPTIONS = [
  { id: 'anthropic', label: 'Anthropic' },
  { id: 'openai', label: 'OpenAI' },
] as const;

@Component({
  selector: 'app-admin-platform-config',
  imports: [KeyValuePipe, ReactiveFormsModule],
  templateUrl: './admin-platform-config.html',
  styleUrl: './admin-platform-config.scss',
})
export class AdminPlatformConfig implements OnInit {
  private readonly api = inject(AdminConfigApiService);

  protected readonly loadState = signal<LoadState>('loading');
  protected readonly loadError = signal<string | null>(null);
  protected readonly actionError = signal<string | null>(null);
  protected readonly saveSuccess = signal<string | null>(null);
  protected readonly saving = signal(false);
  protected readonly providerOptions = PROVIDER_OPTIONS;

  protected readonly form = new FormGroup({
    anthropic: new FormControl(false, { nonNullable: true }),
    openai: new FormControl(false, { nonNullable: true }),
    defaultMonthlyTokenLimit: new FormControl('', { nonNullable: true }),
    platformAnnouncement: new FormControl('', { nonNullable: true }),
    aiRequestsEnabled: new FormControl(true, { nonNullable: true }),
  });

  protected readonly extraFlagKey = new FormControl('', { nonNullable: true });
  protected readonly extraFlagValue = new FormControl(true, { nonNullable: true });
  protected readonly extraFlags = signal<Record<string, boolean>>({});

  ngOnInit(): void {
    this.load();
  }

  protected toggleProvider(id: string, checked: boolean): void {
    if (id === 'anthropic') {
      this.form.controls.anthropic.setValue(checked);
    } else if (id === 'openai') {
      this.form.controls.openai.setValue(checked);
    }
  }

  protected providerChecked(id: string): boolean {
    if (id === 'anthropic') {
      return this.form.controls.anthropic.value;
    }
    if (id === 'openai') {
      return this.form.controls.openai.value;
    }
    return false;
  }

  protected addExtraFlag(): void {
    const key = this.extraFlagKey.value.trim();
    if (!key || !/^[a-zA-Z0-9_]+$/.test(key)) {
      this.actionError.set(
        'Flag keys may only contain letters, numbers, and underscores.',
      );
      return;
    }
    this.extraFlags.update((flags) => ({ ...flags, [key]: this.extraFlagValue.value }));
    this.extraFlagKey.setValue('');
    this.extraFlagValue.setValue(true);
    this.actionError.set(null);
  }

  protected removeExtraFlag(key: string): void {
    this.extraFlags.update((flags) => {
      const next = { ...flags };
      delete next[key];
      return next;
    });
  }

  protected save(): void {
    if (this.saving()) {
      return;
    }

    const allowed: string[] = [];
    if (this.form.controls.anthropic.value) {
      allowed.push('anthropic');
    }
    if (this.form.controls.openai.value) {
      allowed.push('openai');
    }
    if (allowed.length === 0) {
      this.actionError.set('Select at least one allowed AI provider.');
      return;
    }

    const limitRaw = this.form.controls.defaultMonthlyTokenLimit.value.trim();
    let defaultMonthlyTokenLimit: number | null = null;
    if (limitRaw) {
      const parsed = Number(limitRaw);
      if (!Number.isFinite(parsed) || parsed <= 0) {
        this.actionError.set('Default monthly token limit must be a positive number.');
        return;
      }
      defaultMonthlyTokenLimit = parsed;
    }

    const announcementRaw = this.form.controls.platformAnnouncement.value.trim();
    const platformAnnouncement = announcementRaw.length > 0 ? announcementRaw : null;

    const featureFlags: Record<string, boolean> = {
      ...this.extraFlags(),
      ai_requests_enabled: this.form.controls.aiRequestsEnabled.value,
    };

    const confirmed = window.confirm(
      'Save platform configuration? Changes apply on the next request.',
    );
    if (!confirmed) {
      return;
    }

    const body: UpdatePlatformConfigRequest = {
      allowedAiProviders: allowed,
      defaultMonthlyTokenLimit,
      platformAnnouncement,
      featureFlags,
    };

    this.actionError.set(null);
    this.saveSuccess.set(null);
    this.saving.set(true);
    this.api.patch(body).subscribe({
      next: (config) => {
        this.applyConfig(config);
        this.saving.set(false);
        this.saveSuccess.set('Configuration saved.');
      },
      error: (err: unknown) => {
        this.actionError.set(AdminConfigApiService.errorMessage(err));
        this.saving.set(false);
      },
    });
  }

  protected load(): void {
    this.loadState.set('loading');
    this.loadError.set(null);
    this.api.get().subscribe({
      next: (config) => {
        this.applyConfig(config);
        this.loadState.set('ok');
      },
      error: (err: unknown) => {
        this.loadError.set(AdminConfigApiService.errorMessage(err));
        this.loadState.set('error');
      },
    });
  }

  private applyConfig(config: PlatformConfig): void {
    this.form.patchValue({
      anthropic: config.allowedAiProviders.includes('anthropic'),
      openai: config.allowedAiProviders.includes('openai'),
      defaultMonthlyTokenLimit:
        config.defaultMonthlyTokenLimit != null
          ? String(config.defaultMonthlyTokenLimit)
          : '',
      platformAnnouncement: config.platformAnnouncement ?? '',
      aiRequestsEnabled: config.featureFlags['ai_requests_enabled'] ?? true,
    });

    const extras: Record<string, boolean> = {};
    for (const [key, value] of Object.entries(config.featureFlags)) {
      if (key !== 'ai_requests_enabled') {
        extras[key] = value;
      }
    }
    this.extraFlags.set(extras);
  }
}
