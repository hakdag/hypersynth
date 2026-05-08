import { CommonModule } from '@angular/common';
import { Component, inject, OnDestroy, OnInit, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { catchError, finalize, map, of, Subscription, switchMap } from 'rxjs';

import {
  ProjectApiService,
  ProjectDetail as ProjectDetailModel,
} from '../project-api.service';

type LoadState = 'idle' | 'loading' | 'ok' | 'error';

const STATUSES = ['Pending', 'In Progress', 'Done'] as const;
type ProjectPhase0Status = (typeof STATUSES)[number];

@Component({
  selector: 'app-project-edit',
  imports: [CommonModule, ReactiveFormsModule, RouterLink],
  templateUrl: './project-edit.html',
  styleUrl: './project-edit.scss',
})
export class ProjectEdit implements OnInit, OnDestroy {
  private readonly fb = inject(FormBuilder);
  private readonly projectApi = inject(ProjectApiService);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private sub: Subscription | null = null;

  protected readonly loadState = signal<LoadState>('loading');
  protected readonly loadError = signal<string | null>(null);
  protected readonly projectId = signal('');
  protected readonly headerName = signal('');
  protected readonly hadAiApiKey = signal(false);
  protected readonly submitting = signal(false);
  protected readonly serverError = signal<string | null>(null);
  protected readonly showSuccess = signal(false);
  protected readonly apiKeyVisible = signal(false);
  protected readonly enhancing = signal(false);
  protected readonly enhanceError = signal<string | null>(null);
  protected readonly draftRequirements = signal<string | null>(null);
  protected readonly originalRequirements = signal('');

  protected readonly statuses = STATUSES;

  protected readonly form = this.fb.nonNullable.group({
    name: ['', [Validators.required, Validators.maxLength(512)]],
    requirements: [''],
    status: ['Pending' as ProjectPhase0Status, Validators.required],
    aiApiKey: [''],
    clearAiApiKey: [false],
  });

  ngOnInit(): void {
    this.sub = this.route.paramMap
      .pipe(
        switchMap((params) => {
          const id = params.get('projectId') ?? '';
          if (!id) {
            return of<
              | { kind: 'invalid' }
              | { kind: 'ok'; row: ProjectDetailModel }
              | { kind: 'error'; message: string }
            >({ kind: 'invalid' });
          }
          this.projectId.set(id);
          this.loadState.set('loading');
          this.loadError.set(null);
          return this.projectApi.getProject(id).pipe(
            map((row) => ({ kind: 'ok' as const, row })),
            catchError((err: unknown) =>
              of({
                kind: 'error' as const,
                message: ProjectApiService.detailErrorMessage(err),
              }),
            ),
          );
        }),
      )
      .subscribe((res) => {
        if (res.kind === 'invalid') {
          this.loadError.set('No project identifier provided.');
          this.loadState.set('error');
          return;
        }
        if (res.kind === 'error') {
          this.loadError.set(res.message);
          this.loadState.set('error');
          return;
        }
        const p = res.row;
        this.headerName.set(p.name);
        this.hadAiApiKey.set(p.hasAiApiKey);
        this.form.patchValue({
          name: p.name,
          requirements: p.requirements ?? '',
          status: this.normalizeStatus(p.status),
          aiApiKey: '',
          clearAiApiKey: false,
        });
        this.loadState.set('ok');
      });
  }

  ngOnDestroy(): void {
    this.sub?.unsubscribe();
  }

  private normalizeStatus(s: string): ProjectPhase0Status {
    if (s === 'In Progress' || s === 'Done' || s === 'Pending') {
      return s;
    }
    return 'Pending';
  }

  protected toggleApiKeyVisible(): void {
    this.apiKeyVisible.update((v) => !v);
  }

  protected onClearKeyChecked(checked: boolean): void {
    if (checked) {
      this.form.patchValue({ aiApiKey: '' });
    }
  }

  protected submit(): void {
    this.serverError.set(null);
    this.form.markAllAsTouched();
    if (this.form.invalid || this.submitting() || this.loadState() !== 'ok') {
      return;
    }

    const id = this.projectId();
    const { name, requirements, status, aiApiKey, clearAiApiKey } = this.form.getRawValue();

    this.submitting.set(true);
    this.projectApi
      .updateProject(id, {
        name: name.trim(),
        requirements,
        status,
        clearAiApiKey,
        aiApiKey,
      })
      .pipe(finalize(() => this.submitting.set(false)))
      .subscribe({
        next: () => {
          this.showSuccess.set(true);
        },
        error: (err: unknown) => {
          this.serverError.set(ProjectApiService.updateErrorMessage(err));
        },
      });
  }

  protected canEnhanceRequirements(): boolean {
    if (!this.hadAiApiKey()) {
      return false;
    }
    if (this.submitting() || this.enhancing() || this.loadState() !== 'ok') {
      return false;
    }
    const requirements = this.form.controls.requirements.value ?? '';
    return requirements.trim().length > 0;
  }

  protected enhanceRequirementsDisabledReason(): string {
    if (!this.hadAiApiKey()) {
      return 'Add an AI API key to use this.';
    }
    const requirements = this.form.controls.requirements.value ?? '';
    if (requirements.trim().length === 0) {
      return 'Add project requirements first.';
    }
    return '';
  }

  protected enhanceWithAi(): void {
    this.enhanceError.set(null);
    if (!this.canEnhanceRequirements()) {
      const reason = this.enhanceRequirementsDisabledReason();
      if (reason.length > 0) {
        this.enhanceError.set(reason);
      }
      return;
    }

    const id = this.projectId();
    const currentRequirements = this.form.controls.requirements.value ?? '';
    this.originalRequirements.set(currentRequirements);
    this.enhancing.set(true);
    this.projectApi
      .enhanceProjectRequirements(id)
      .pipe(finalize(() => this.enhancing.set(false)))
      .subscribe({
        next: (result) => {
          this.draftRequirements.set(result.enhancedRequirements);
        },
        error: (err: unknown) => {
          this.enhanceError.set(ProjectApiService.enhanceErrorMessage(err));
        },
      });
  }

  protected acceptDraft(): void {
    const draft = this.draftRequirements();
    if (!draft) {
      return;
    }
    this.form.patchValue({ requirements: draft });
    this.form.controls.requirements.markAsDirty();
    this.draftRequirements.set(null);
    this.originalRequirements.set('');
  }

  protected rejectDraft(): void {
    this.draftRequirements.set(null);
    this.originalRequirements.set('');
  }

  protected cancel(): void {
    const id = this.projectId();
    if (id.length > 0) {
      void this.router.navigateByUrl(`/app/projects/${encodeURIComponent(id)}`);
    } else {
      void this.router.navigateByUrl('/app/projects');
    }
  }

  protected dismissSuccessToDetail(): void {
    this.showSuccess.set(false);
    const id = this.projectId();
    void this.router.navigateByUrl(`/app/projects/${encodeURIComponent(id)}`);
  }

  protected nameError(): string {
    const c = this.form.controls.name;
    if (!c.touched || !c.errors) {
      return '';
    }
    if (c.errors['required']) {
      return 'Project name is required.';
    }
    if (c.errors['maxlength']) {
      return 'Name is too long.';
    }
    return '';
  }
}
