import { CommonModule } from '@angular/common';
import {
  Component,
  computed,
  inject,
  OnDestroy,
  OnInit,
  signal,
} from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import {
  catchError,
  finalize,
  forkJoin,
  map,
  of,
  Subscription,
  switchMap,
} from 'rxjs';

import {
  CreatedFeature,
  ProjectApiService,
  ProjectDetail as ProjectDetailModel,
} from '../project-api.service';

const VALID_FEATURE_STATUSES = ['Pending', 'In Progress', 'Done'] as const;
type ValidFeatureStatus = (typeof VALID_FEATURE_STATUSES)[number];

function normalizeStatus(raw: string): string {
  return VALID_FEATURE_STATUSES.includes(raw as ValidFeatureStatus) ? raw : 'Pending';
}

type PageResult =
  | { kind: 'invalid' }
  | { kind: 'ok'; project: ProjectDetailModel; feature: CreatedFeature }
  | { kind: 'error'; message: string };

@Component({
  selector: 'app-feature-detail',
  imports: [CommonModule, ReactiveFormsModule, RouterLink],
  templateUrl: './feature-detail.html',
  styleUrls: ['./feature-detail.scss', '../feature-create/feature-create.scss'],
})
export class FeatureDetail implements OnInit, OnDestroy {
  private readonly fb = inject(FormBuilder);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private readonly projectApi = inject(ProjectApiService);
  private sub: Subscription | null = null;

  protected readonly loadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly project = signal<ProjectDetailModel | null>(null);
  protected readonly featureMeta = signal<CreatedFeature | null>(null);
  protected readonly pageError = signal<string | null>(null);

  protected readonly submitting = signal(false);
  protected readonly serverError = signal<string | null>(null);
  protected readonly saveNotice = signal(false);
  protected readonly enhancing = signal(false);
  protected readonly enhanceError = signal<string | null>(null);
  protected readonly draftRequirements = signal<string | null>(null);
  protected readonly originalRequirements = signal('');
  protected readonly requirementsCopyFlash = signal(false);

  private requirementsCopyTimer: ReturnType<typeof setTimeout> | null = null;

  /** Local-only status change before Save; cleared on load and after successful PATCH. */
  private readonly statusLocalOverride = signal<string | null>(null);

  protected readonly effectiveStatus = computed(() => {
    const m = this.featureMeta();
    if (!m) {
      return 'Pending';
    }
    const o = this.statusLocalOverride();
    return normalizeStatus(o ?? m.status);
  });

  protected readonly form = this.fb.nonNullable.group({
    title: ['', [Validators.required, Validators.maxLength(512)]],
    requirements: [''],
  });

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
          this.serverError.set(null);
          this.saveNotice.set(false);
          this.statusLocalOverride.set(null);
          this.clearRequirementsCopyFlash();
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
          this.featureMeta.set(null);
          this.statusLocalOverride.set(null);
          this.loadState.set('error');
          return;
        }
        if (res.kind === 'error') {
          this.pageError.set(res.message);
          this.project.set(null);
          this.featureMeta.set(null);
          this.statusLocalOverride.set(null);
          this.loadState.set('error');
          return;
        }
        this.project.set(res.project);
        this.featureMeta.set(res.feature);
        this.pageError.set(null);
        this.form.patchValue({
          title: res.feature.title,
          requirements: res.feature.requirements ?? '',
        });
        this.form.markAsPristine();
        this.saveNotice.set(false);
        this.statusLocalOverride.set(null);
        this.loadState.set('ok');
      });
  }

  ngOnDestroy(): void {
    this.sub?.unsubscribe();
    this.clearRequirementsCopyFlash();
  }

  private clearRequirementsCopyFlash(): void {
    if (this.requirementsCopyTimer !== null) {
      clearTimeout(this.requirementsCopyTimer);
      this.requirementsCopyTimer = null;
    }
    this.requirementsCopyFlash.set(false);
  }

  protected copyRequirementsFromForm(): void {
    const text = this.form.controls.requirements.value ?? '';
    void navigator.clipboard.writeText(text).then(() => {
      if (this.requirementsCopyTimer !== null) {
        clearTimeout(this.requirementsCopyTimer);
        this.requirementsCopyTimer = null;
      }
      this.requirementsCopyFlash.set(true);
      this.requirementsCopyTimer = setTimeout(() => {
        this.requirementsCopyFlash.set(false);
        this.requirementsCopyTimer = null;
      }, 1600);
    });
  }

  protected startProgress(): void {
    if (this.effectiveStatus() !== 'Pending') {
      return;
    }
    this.statusLocalOverride.set('In Progress');
  }

  protected submit(): void {
    const p = this.project();
    const f = this.featureMeta();
    if (!p || !f) return;

    this.serverError.set(null);
    this.saveNotice.set(false);
    this.form.markAllAsTouched();
    if (this.form.invalid || this.submitting()) {
      return;
    }

    const raw = this.form.getRawValue();
    this.submitting.set(true);

    this.projectApi
      .updateFeature(p.id, f.id, {
        title: raw.title.trim(),
        requirements: raw.requirements.trim().length > 0 ? raw.requirements : '',
        status: normalizeStatus(this.effectiveStatus()),
      })
      .pipe(finalize(() => this.submitting.set(false)))
      .subscribe({
        next: (updated) => {
          this.statusLocalOverride.set(null);
          this.featureMeta.set(updated);
          this.form.patchValue({
            title: updated.title,
            requirements: updated.requirements ?? '',
          });
          this.form.markAsPristine();
          this.saveNotice.set(true);
        },
        error: (err: unknown) => {
          this.serverError.set(ProjectApiService.updateFeatureErrorMessage(err));
        },
      });
  }

  protected cancel(): void {
    const p = this.project();
    if (p) {
      void this.router.navigateByUrl(`/app/projects/${encodeURIComponent(p.id)}`);
    } else {
      void this.router.navigateByUrl('/app/projects');
    }
  }

  protected titleError(): string {
    const c = this.form.controls.title;
    if (!c.touched || !c.errors) {
      return '';
    }
    if (c.errors['required']) {
      return 'Feature title is required.';
    }
    if (c.errors['maxlength']) {
      return 'Title is too long.';
    }
    return '';
  }

  protected shortFeatureId(id: string): string {
    return id.replace(/-/g, '').slice(0, 8).toUpperCase();
  }

  protected canEnhanceFeatureRequirements(): boolean {
    const p = this.project();
    if (!p?.hasAiApiKey) {
      return false;
    }
    if (this.submitting() || this.enhancing() || this.loadState() !== 'ok') {
      return false;
    }
    const requirements = this.form.controls.requirements.value ?? '';
    return requirements.trim().length > 0;
  }

  protected enhanceFeatureRequirementsDisabledReason(): string {
    const p = this.project();
    if (!p?.hasAiApiKey) {
      return 'Add an AI API key to the project to use this.';
    }
    const requirements = this.form.controls.requirements.value ?? '';
    if (requirements.trim().length === 0) {
      return 'Add feature requirements first.';
    }
    return '';
  }

  protected enhanceFeatureWithAi(): void {
    this.enhanceError.set(null);
    if (!this.canEnhanceFeatureRequirements()) {
      const reason = this.enhanceFeatureRequirementsDisabledReason();
      if (reason.length > 0) {
        this.enhanceError.set(reason);
      }
      return;
    }

    const p = this.project();
    const f = this.featureMeta();
    if (!p || !f) {
      return;
    }

    const currentRequirements = this.form.controls.requirements.value ?? '';
    this.originalRequirements.set(currentRequirements);
    this.enhancing.set(true);
    this.projectApi
      .enhanceFeatureRequirements(p.id, f.id)
      .pipe(finalize(() => this.enhancing.set(false)))
      .subscribe({
        next: (result) => {
          this.draftRequirements.set(result.enhancedRequirements);
        },
        error: (err: unknown) => {
          this.enhanceError.set(ProjectApiService.enhanceFeatureRequirementsErrorMessage(err));
        },
      });
  }

  protected acceptFeatureDraft(): void {
    const draft = this.draftRequirements();
    if (!draft) {
      return;
    }
    this.form.patchValue({ requirements: draft });
    this.form.controls.requirements.markAsDirty();
    this.draftRequirements.set(null);
    this.originalRequirements.set('');
  }

  protected rejectFeatureDraft(): void {
    this.draftRequirements.set(null);
    this.originalRequirements.set('');
  }
}
