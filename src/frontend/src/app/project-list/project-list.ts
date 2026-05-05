import { Component, computed, inject, OnInit, signal } from '@angular/core';
import { RouterLink } from '@angular/router';

import { CreatedProject, ProjectApiService } from '../project-api.service';

@Component({
  selector: 'app-project-list',
  imports: [RouterLink],
  templateUrl: './project-list.html',
  styleUrl: './project-list.scss',
})
export class ProjectList implements OnInit {
  private readonly projectApi = inject(ProjectApiService);

  protected readonly projects = signal<CreatedProject[]>([]);
  protected readonly loadState = signal<'idle' | 'loading' | 'ok' | 'error'>('loading');
  protected readonly listError = signal<string | null>(null);

  protected readonly statusCounts = computed(() => {
    const list = this.projects();
    let pending = 0;
    let inProgress = 0;
    let done = 0;
    for (const p of list) {
      if (p.status === 'Pending') pending += 1;
      else if (p.status === 'In Progress') inProgress += 1;
      else if (p.status === 'Done') done += 1;
    }
    return { pending, inProgress, done, total: list.length };
  });

  protected readonly portfolioPercent = computed(() => {
    const { total, inProgress, done } = this.statusCounts();
    if (total === 0) return 0;
    return Math.round(((inProgress + done) / total) * 100);
  });

  ngOnInit(): void {
    this.loadProjects();
  }

  protected loadProjects(): void {
    this.listError.set(null);
    this.loadState.set('loading');
    this.projectApi.listProjects().subscribe({
      next: (rows) => {
        this.projects.set(rows);
        this.loadState.set('ok');
      },
      error: (err: unknown) => {
        this.loadState.set('error');
        this.listError.set(ProjectApiService.listErrorMessage(err));
      },
    });
  }

  protected statusIcon(status: string): string {
    switch (status) {
      case 'In Progress':
        return 'architecture';
      case 'Done':
        return 'corporate_fare';
      default:
        return 'folder_special';
    }
  }

  protected statusBadgeClass(status: string): string {
    switch (status) {
      case 'In Progress':
        return 'pl-status pl-status--progress';
      case 'Done':
        return 'pl-status pl-status--done';
      default:
        return 'pl-status pl-status--pending';
    }
  }

  protected iconWrapClass(status: string): string {
    switch (status) {
      case 'In Progress':
        return 'pl-card__icon pl-card__icon--progress';
      case 'Done':
        return 'pl-card__icon pl-card__icon--done';
      default:
        return 'pl-card__icon pl-card__icon--pending';
    }
  }

  protected formatCreatedLabel(createdAt: string): string {
    const created = new Date(createdAt);
    if (Number.isNaN(created.getTime())) return 'Created recently';
    const now = Date.now();
    const diffMs = now - created.getTime();
    const days = Math.floor(diffMs / (24 * 60 * 60 * 1000));
    if (days <= 0) return 'Created today';
    if (days === 1) return 'Created yesterday';
    if (days < 14) return `Created ${days} days ago`;
    return `Created ${created.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })}`;
  }

  protected requirementsPreview(requirements: string | null): string {
    if (!requirements || requirements.trim().length === 0) {
      return 'No requirements recorded yet.';
    }
    const t = requirements.trim();
    if (t.length <= 220) return t;
    return `${t.slice(0, 217)}…`;
  }
}
