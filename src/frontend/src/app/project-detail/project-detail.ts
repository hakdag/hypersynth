import { Component, inject, OnDestroy, OnInit, signal } from '@angular/core';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { catchError, map, of, Subscription, switchMap } from 'rxjs';

import { ProjectApiService, ProjectDetail as ProjectDetailModel } from '../project-api.service';

type DetailResult =
  | { kind: 'invalid' }
  | { kind: 'ok'; row: ProjectDetailModel }
  | { kind: 'error'; message: string };

@Component({
  selector: 'app-project-detail',
  imports: [RouterLink],
  templateUrl: './project-detail.html',
  styleUrl: './project-detail.scss',
})
export class ProjectDetail implements OnInit, OnDestroy {
  private readonly route = inject(ActivatedRoute);
  private readonly projectApi = inject(ProjectApiService);
  private sub: Subscription | null = null;

  protected readonly loadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly project = signal<ProjectDetailModel | null>(null);
  protected readonly detailError = signal<string | null>(null);

  private readonly rtf = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' });

  ngOnInit(): void {
    this.sub = this.route.paramMap
      .pipe(
        switchMap((params) => {
          const id = params.get('projectId') ?? '';
          if (id.length === 0) {
            return of<DetailResult>({ kind: 'invalid' });
          }
          this.loadState.set('loading');
          this.detailError.set(null);
          return this.projectApi.getProject(id).pipe(
            map((row): DetailResult => ({ kind: 'ok', row })),
            catchError((err: unknown) =>
              of<DetailResult>({
                kind: 'error',
                message: ProjectApiService.detailErrorMessage(err),
              }),
            ),
          );
        }),
      )
      .subscribe((res) => {
        if (res.kind === 'invalid') {
          this.detailError.set('No project identifier provided.');
          this.project.set(null);
          this.loadState.set('error');
          return;
        }
        if (res.kind === 'error') {
          this.detailError.set(res.message);
          this.project.set(null);
          this.loadState.set('error');
          return;
        }
        this.project.set(res.row);
        this.detailError.set(null);
        this.loadState.set('ok');
      });
  }

  ngOnDestroy(): void {
    this.sub?.unsubscribe();
  }

  protected completionPercent(status: string): number {
    switch (status) {
      case 'In Progress':
        return 50;
      case 'Done':
        return 100;
      default:
        return 0;
    }
  }

  protected priorityLabel(status: string): string {
    switch (status) {
      case 'In Progress':
        return 'Elevated';
      case 'Done':
        return 'Complete';
      default:
        return 'Standard';
    }
  }

  protected priorityIcon(status: string): string {
    switch (status) {
      case 'In Progress':
        return 'priority_high';
      case 'Done':
        return 'check_circle';
      default:
        return 'flag';
    }
  }

  protected priorityIconClass(status: string): string {
    const base = 'material-symbols-outlined pd-priority__icon';
    switch (status) {
      case 'In Progress':
        return `${base} pd-priority__icon--high`;
      case 'Done':
        return `${base} pd-priority__icon--done`;
      default:
        return `${base} pd-priority__icon--std`;
    }
  }

  protected tagline(requirements: string | null): string {
    const t = requirements?.trim();
    if (t && t.length > 0) {
      if (t.length <= 200) return t;
      return `${t.slice(0, 197)}…`;
    }
    return 'Add requirements when creating or editing the project.';
  }

  protected relativeCreatedLabel(iso: string): string {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return '—';
    const diffSec = Math.round((d.getTime() - Date.now()) / 1000);
    const abs = Math.abs(diffSec);
    if (abs < 45) return 'Created just now';
    const divisions: { unit: Intl.RelativeTimeFormatUnit; secs: number }[] = [
      { unit: 'year', secs: 31536000 },
      { unit: 'month', secs: 2592000 },
      { unit: 'week', secs: 604800 },
      { unit: 'day', secs: 86400 },
      { unit: 'hour', secs: 3600 },
      { unit: 'minute', secs: 60 },
    ];
    for (const { unit, secs } of divisions) {
      if (abs >= secs) {
        const delta = Math.trunc(diffSec / secs);
        const rel = this.rtf.format(delta, unit);
        return `Created ${rel}`;
      }
    }
    return `Created ${d.toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' })}`;
  }

  protected statusBadgeClass(status: string): string {
    switch (status) {
      case 'In Progress':
        return 'pd-status pd-status--progress';
      case 'Done':
        return 'pd-status pd-status--done';
      default:
        return 'pd-status pd-status--pending';
    }
  }

  protected statusForDisplay(status: string): string {
    return status.toUpperCase();
  }
}
