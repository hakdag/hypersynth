import { CommonModule } from '@angular/common';
import { Component, inject, OnDestroy, OnInit, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { catchError, finalize, map, of, Subscription, switchMap } from 'rxjs';

import {
  ProjectApiService,
  ProjectDetail as ProjectDetailModel,
} from '../project-api.service';

type PageLoadResult =
  | { kind: 'invalid' }
  | { kind: 'ok'; row: ProjectDetailModel }
  | { kind: 'error'; message: string };

@Component({
  selector: 'app-feature-create',
  imports: [CommonModule, ReactiveFormsModule, RouterLink],
  templateUrl: './feature-create.html',
  styleUrl: './feature-create.scss',
})
export class FeatureCreate implements OnInit, OnDestroy {
  private readonly fb = inject(FormBuilder);
  private readonly projectApi = inject(ProjectApiService);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private sub: Subscription | null = null;

  protected readonly loadState = signal<'loading' | 'ok' | 'error'>('loading');
  protected readonly project = signal<ProjectDetailModel | null>(null);
  protected readonly pageError = signal<string | null>(null);

  protected readonly submitting = signal(false);
  protected readonly serverError = signal<string | null>(null);
  protected readonly showSuccess = signal(false);

  protected readonly form = this.fb.nonNullable.group({
    title: ['', [Validators.required, Validators.maxLength(512)]],
    requirements: [''],
  });

  ngOnInit(): void {
    this.sub = this.route.paramMap
      .pipe(
        switchMap((params) => {
          const id = params.get('projectId') ?? '';
          if (id.length === 0) {
            return of<PageLoadResult>({ kind: 'invalid' });
          }
          this.loadState.set('loading');
          this.pageError.set(null);
          return this.projectApi.getProject(id).pipe(
            map((row): PageLoadResult => ({ kind: 'ok', row })),
            catchError((err: unknown) =>
              of<PageLoadResult>({
                kind: 'error',
                message: ProjectApiService.detailErrorMessage(err),
              }),
            ),
          );
        }),
      )
      .subscribe((res) => {
        if (res.kind === 'invalid') {
          this.pageError.set('No project identifier provided.');
          this.project.set(null);
          this.loadState.set('error');
          return;
        }
        if (res.kind === 'error') {
          this.pageError.set(res.message);
          this.project.set(null);
          this.loadState.set('error');
          return;
        }
        this.project.set(res.row);
        this.pageError.set(null);
        this.loadState.set('ok');
      });
  }

  ngOnDestroy(): void {
    this.sub?.unsubscribe();
  }

  protected submit(): void {
    const p = this.project();
    if (!p) return;

    this.serverError.set(null);
    this.form.markAllAsTouched();
    if (this.form.invalid || this.submitting()) {
      return;
    }

    const { title, requirements } = this.form.getRawValue();
    this.submitting.set(true);

    this.projectApi
      .createFeature(p.id, {
        title: title.trim(),
        requirements: requirements.trim().length > 0 ? requirements : undefined,
      })
      .pipe(finalize(() => this.submitting.set(false)))
      .subscribe({
        next: () => {
          this.showSuccess.set(true);
        },
        error: (err: unknown) => {
          this.serverError.set(ProjectApiService.createFeatureErrorMessage(err));
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

  protected dismissSuccess(): void {
    const p = this.project();
    this.showSuccess.set(false);
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
}
