import { Component, inject, OnDestroy, OnInit, signal } from '@angular/core';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { catchError, forkJoin, map, of, Subscription, switchMap } from 'rxjs';

import {
  CreatedFeature,
  ProjectApiService,
  ProjectDetail as ProjectDetailModel,
} from '../project-api.service';

type PageResult =
  | { kind: 'invalid' }
  | { kind: 'ok'; project: ProjectDetailModel; feature: CreatedFeature }
  | { kind: 'error'; message: string };

@Component({
  selector: 'app-feature-detail',
  imports: [RouterLink],
  templateUrl: './feature-detail.html',
  styleUrl: './feature-detail.scss',
})
export class FeatureDetail implements OnInit, OnDestroy {
  private readonly route = inject(ActivatedRoute);
  private readonly projectApi = inject(ProjectApiService);
  private sub: Subscription | null = null;

  protected readonly loadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly project = signal<ProjectDetailModel | null>(null);
  protected readonly feature = signal<CreatedFeature | null>(null);
  protected readonly pageError = signal<string | null>(null);

  protected readonly requirementsExpanded = signal(false);

  ngOnInit(): void {
    this.sub = this.route.paramMap
      .pipe(
        switchMap((params) => {
          const projectId = params.get('projectId') ?? '';
          const featureId = params.get('featureId') ?? '';
          if (projectId.length === 0 || featureId.length === 0) {
            return of<PageResult>({ kind: 'invalid' });
          }
          this.loadState.set('loading');
          this.pageError.set(null);
          return forkJoin({
            project: this.projectApi.getProject(projectId),
            feature: this.projectApi.getFeature(projectId, featureId),
          }).pipe(
            map(
              (data): PageResult => ({
                kind: 'ok',
                project: data.project,
                feature: data.feature,
              }),
            ),
            catchError((err: unknown) =>
              of<PageResult>({
                kind: 'error',
                message: ProjectApiService.featureDetailErrorMessage(err),
              }),
            ),
          );
        }),
      )
      .subscribe((res) => {
        if (res.kind === 'invalid') {
          this.pageError.set('Missing project or feature identifier.');
          this.project.set(null);
          this.feature.set(null);
          this.loadState.set('error');
          this.requirementsExpanded.set(false);
          return;
        }
        if (res.kind === 'error') {
          this.pageError.set(res.message);
          this.project.set(null);
          this.feature.set(null);
          this.loadState.set('error');
          this.requirementsExpanded.set(false);
          return;
        }
        this.project.set(res.project);
        this.feature.set(res.feature);
        this.pageError.set(null);
        this.loadState.set('ok');
        this.requirementsExpanded.set(false);
      });
  }

  ngOnDestroy(): void {
    this.sub?.unsubscribe();
  }

  protected toggleRequirementsExpanded(): void {
    this.requirementsExpanded.update((v) => !v);
  }

  protected requirementsText(): string {
    const r = this.feature()?.requirements?.trim();
    return r && r.length > 0 ? r : '';
  }

  protected hasExpandableRequirements(): boolean {
    return this.requirementsText().length > 0;
  }

  protected statusBadgeClass(status: string): string {
    switch (status) {
      case 'In Progress':
        return 'fd-status fd-status--progress';
      case 'Done':
        return 'fd-status fd-status--done';
      default:
        return 'fd-status fd-status--pending';
    }
  }

  protected statusForDisplay(status: string): string {
    return status.toUpperCase();
  }

  protected shortFeatureId(id: string): string {
    return id.replace(/-/g, '').slice(0, 8).toUpperCase();
  }
}
