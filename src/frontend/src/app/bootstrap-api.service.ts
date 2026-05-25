import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';
import { environment } from '../environments/environment';

export interface BootstrapPayload {
  appName: string;
  statusLabels: string[];
  platformAnnouncement?: string | null;
  featureFlags?: Record<string, boolean>;
}

export type BootstrapLoadState = 'idle' | 'loading' | 'success' | 'error';

/** Used only when the bootstrap request fails — matches backend Phase 0 copy. */
export const PHASE0_STATUS_LABELS_FALLBACK = ['Pending', 'In Progress', 'Done'];

@Injectable({
  providedIn: 'root',
})
export class BootstrapApiService {
  private readonly http = inject(HttpClient);

  readonly loadState = signal<BootstrapLoadState>('idle');
  readonly bootstrap = signal<BootstrapPayload | null>(null);
  readonly lastError = signal<string | null>(null);

  readonly statusLabels = computed(() => {
    const labels = this.bootstrap()?.statusLabels;
    return labels?.length ? labels : PHASE0_STATUS_LABELS_FALLBACK;
  });

  readonly appName = computed(() => this.bootstrap()?.appName ?? 'HyperSynth');

  readonly platformAnnouncement = computed(() => {
    const msg = this.bootstrap()?.platformAnnouncement?.trim();
    return msg && msg.length > 0 ? msg : null;
  });

  readonly featureFlags = computed(
    () => this.bootstrap()?.featureFlags ?? ({} as Record<string, boolean>),
  );

  loadBootstrap(): void {
    this.loadState.set('loading');
    this.lastError.set(null);
    const url = `${environment.apiBaseUrl}/api/v1/bootstrap`;
    this.http.get<BootstrapPayload>(url).subscribe({
      next: (payload) => {
        this.bootstrap.set(payload);
        this.loadState.set('success');
      },
      error: (err: unknown) => {
        this.bootstrap.set(null);
        this.loadState.set('error');
        this.lastError.set(this.formatError(err));
      },
    });
  }

  formatError(err: unknown): string {
    if (err instanceof HttpErrorResponse) {
      const detail = typeof err.error === 'string' ? err.error : err.statusText;
      return `HTTP ${err.status}: ${detail}`;
    }
    return 'Could not reach the server. Ensure the backend is running.';
  }
}
