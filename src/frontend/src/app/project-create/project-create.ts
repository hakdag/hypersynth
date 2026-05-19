import { CommonModule } from '@angular/common';
import { Component, inject, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { Router, RouterLink } from '@angular/router';
import { finalize } from 'rxjs';

import { ProjectApiService } from '../project-api.service';

@Component({
  selector: 'app-project-create',
  imports: [CommonModule, ReactiveFormsModule, RouterLink],
  templateUrl: './project-create.html',
  styleUrl: './project-create.scss',
})
export class ProjectCreate {
  private readonly fb = inject(FormBuilder);
  private readonly projectApi = inject(ProjectApiService);
  private readonly router = inject(Router);

  protected readonly submitting = signal(false);
  protected readonly serverError = signal<string | null>(null);
  protected readonly showSuccess = signal(false);

  protected readonly form = this.fb.nonNullable.group({
    name: ['', [Validators.required, Validators.maxLength(512)]],
    requirements: [''],
  });

  protected submit(): void {
    this.serverError.set(null);
    this.form.markAllAsTouched();
    if (this.form.invalid || this.submitting()) {
      return;
    }

    const { name, requirements } = this.form.getRawValue();
    this.submitting.set(true);

    this.projectApi
      .createProject({
        name: name.trim(),
        requirements: requirements.trim().length > 0 ? requirements : undefined,
      })
      .pipe(finalize(() => this.submitting.set(false)))
      .subscribe({
        next: () => {
          this.showSuccess.set(true);
        },
        error: (err: unknown) => {
          this.serverError.set(ProjectApiService.errorMessage(err));
        },
      });
  }

  protected cancel(): void {
    void this.router.navigateByUrl('/app/projects');
  }

  protected dismissSuccessToProjects(): void {
    this.showSuccess.set(false);
    void this.router.navigateByUrl('/app/projects');
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
