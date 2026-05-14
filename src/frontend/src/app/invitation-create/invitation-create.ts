import { CommonModule } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { Router, RouterLink } from '@angular/router';
import { finalize } from 'rxjs';

import type { CompanyRole } from '../auth-api.service';
import { AuthService } from '../auth.service';
import { InvitationApiService } from '../invitation-api.service';
import { ProjectApiService } from '../project-api.service';

@Component({
  selector: 'app-invitation-create',
  imports: [CommonModule, ReactiveFormsModule, RouterLink],
  templateUrl: './invitation-create.html',
  styleUrl: './invitation-create.scss',
})
export class InvitationCreate implements OnInit {
  private readonly fb = inject(FormBuilder);
  private readonly invitationApi = inject(InvitationApiService);
  private readonly projectApi = inject(ProjectApiService);
  private readonly auth = inject(AuthService);
  private readonly router = inject(Router);

  protected readonly submitting = signal(false);
  protected readonly serverError = signal<string | null>(null);
  protected readonly projects = signal<{ id: string; name: string }[]>([]);
  protected readonly projectsLoadError = signal<string | null>(null);

  protected readonly form = this.fb.nonNullable.group({
    invitedEmail: ['', [Validators.required, Validators.email]],
    invitedRole: this.fb.nonNullable.control<CompanyRole>('contributor', Validators.required),
    projectId: [''],
    message: [''],
  });

  ngOnInit(): void {
    this.projectApi.listProjects().subscribe({
      next: (rows) => {
        this.projects.set(rows.map((p) => ({ id: p.id, name: p.name })));
      },
      error: () => {
        this.projectsLoadError.set('Could not load projects for optional assignment.');
      },
    });
  }

  protected roleOptions(): { value: CompanyRole; label: string }[] {
    const u = this.auth.currentUser();
    if (u?.role === 'company_admin') {
      return [
        { value: 'company_admin', label: 'Company Admin' },
        { value: 'project_manager', label: 'Project Manager' },
        { value: 'contributor', label: 'Contributor' },
        { value: 'viewer', label: 'Viewer' },
      ];
    }
    return [
      { value: 'contributor', label: 'Contributor' },
      { value: 'viewer', label: 'Viewer' },
    ];
  }

  protected submit(): void {
    this.serverError.set(null);
    this.form.markAllAsTouched();
    if (this.form.invalid || this.submitting()) {
      return;
    }

    const raw = this.form.getRawValue();
    const projectId = raw.projectId.trim();
    const payload = {
      invitedEmail: raw.invitedEmail.trim(),
      invitedRole: raw.invitedRole,
      projectId: projectId.length > 0 ? projectId : null,
      message: raw.message.trim().length > 0 ? raw.message.trim() : null,
    };

    this.submitting.set(true);
    this.invitationApi
      .createInvitation(payload)
      .pipe(finalize(() => this.submitting.set(false)))
      .subscribe({
        next: () => void this.router.navigateByUrl('/app/team/invitations'),
        error: (err: unknown) => {
          this.serverError.set(InvitationApiService.errorMessage(err));
        },
      });
  }

  protected cancel(): void {
    void this.router.navigateByUrl('/app/team/invitations');
  }

  protected emailError(): string {
    const c = this.form.controls.invitedEmail;
    if (!c.touched || !c.errors) {
      return '';
    }
    if (c.errors['required']) {
      return 'Email is required.';
    }
    if (c.errors['email']) {
      return 'Enter a valid email address.';
    }
    return '';
  }
}
