import { Component, OnInit, inject, signal } from '@angular/core';

import {
  AdminHealthApiService,
  AdminSystemHealthResponse,
  HealthIndicator,
  HealthIndicatorStatus,
} from '../admin-health-api.service';

type LoadState = 'loading' | 'ok' | 'error';

interface HealthCard {
  key: keyof AdminSystemHealthResponse;
  label: string;
  indicator: HealthIndicator;
}

@Component({
  selector: 'app-admin-system-health',
  imports: [],
  templateUrl: './admin-system-health.html',
  styleUrl: './admin-system-health.scss',
})
export class AdminSystemHealth implements OnInit {
  private readonly api = inject(AdminHealthApiService);

  protected readonly loadState = signal<LoadState>('loading');
  protected readonly loadError = signal<string | null>(null);
  protected readonly cards = signal<HealthCard[]>([]);

  ngOnInit(): void {
    this.refresh();
  }

  protected refresh(): void {
    this.loadState.set('loading');
    this.loadError.set(null);
    this.api.getHealth().subscribe({
      next: (health) => {
        this.cards.set(this.buildCards(health));
        this.loadState.set('ok');
      },
      error: (err: unknown) => {
        this.loadError.set(AdminHealthApiService.errorMessage(err));
        this.loadState.set('error');
      },
    });
  }

  protected statusLabel(status: HealthIndicatorStatus): string {
    switch (status) {
      case 'healthy':
        return 'Healthy';
      case 'degraded':
        return 'Degraded';
      case 'unavailable':
        return 'Unavailable';
      case 'not_configured':
        return 'Not configured';
      default:
        return status;
    }
  }

  protected statusClass(status: HealthIndicatorStatus): string {
    switch (status) {
      case 'healthy':
        return 'ash-badge ash-badge-healthy';
      case 'degraded':
        return 'ash-badge ash-badge-degraded';
      case 'unavailable':
        return 'ash-badge ash-badge-unavailable';
      case 'not_configured':
        return 'ash-badge ash-badge-neutral';
      default:
        return 'ash-badge';
    }
  }

  private buildCards(health: AdminSystemHealthResponse): HealthCard[] {
    return [
      { key: 'application', label: 'Application', indicator: health.application },
      { key: 'database', label: 'Database', indicator: health.database },
      { key: 'backgroundJobs', label: 'Background jobs', indicator: health.backgroundJobs },
      {
        key: 'aiProviderErrorRate',
        label: 'AI provider error rate',
        indicator: health.aiProviderErrorRate,
      },
      { key: 'emailDelivery', label: 'Email delivery', indicator: health.emailDelivery },
      { key: 'storage', label: 'Storage', indicator: health.storage },
    ];
  }
}
