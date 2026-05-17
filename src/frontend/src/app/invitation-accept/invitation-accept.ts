import { CommonModule } from '@angular/common';
import {
  Component,
  OnDestroy,
  OnInit,
  inject,
  signal,
} from '@angular/core';
import {
  AbstractControl,
  FormBuilder,
  ReactiveFormsModule,
  ValidationErrors,
  Validators,
} from '@angular/forms';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { Subscription } from 'rxjs';

import { AuthService } from '../auth.service';
import { BootstrapApiService } from '../bootstrap-api.service';
import {
  InvitationApiService,
  type InvitationPreview,
} from '../invitation-api.service';

import type { CompanyRole } from '../auth-api.service';

const USERNAME_PATTERN = /^[a-zA-Z0-9_.-]{3,64}$/;

function phase0PasswordRules(control: AbstractControl): ValidationErrors | null {
  const v = control.value as string | null | undefined;
  if (v === null || v === undefined || v.length === 0) {
    return null;
  }
  const hasLetter = /[a-zA-Z]/.test(v);
  const hasDigit = /\d/.test(v);
  if (!hasLetter || !hasDigit) {
    return { phase0Password: true };
  }
  return null;
}

function passwordConfirmationMatch(group: AbstractControl): ValidationErrors | null {
  const password = group.get('password')?.value as string | null | undefined;
  const confirmation = group.get('passwordConfirmation')?.value as string | null | undefined;
  if (password === null || password === undefined || password.length === 0) {
    return null;
  }
  if (confirmation === null || confirmation === undefined || confirmation.length === 0) {
    return null;
  }
  if (password !== confirmation) {
    return { passwordMismatch: true };
  }
  return null;
}

type UiPhase =
  | 'loading_preview'
  | 'preview_error'
  | 'missing_token'
  | 'register'
  | 'login_required'
  | 'wrong_account'
  | 'confirm';

@Component({
  selector: 'app-invitation-accept',
  imports: [CommonModule, ReactiveFormsModule, RouterLink],
  templateUrl: './invitation-accept.html',
  styleUrl: './invitation-accept.scss',
})
export class InvitationAccept implements OnInit, OnDestroy {
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private readonly fb = inject(FormBuilder);
  private readonly invitationApi = inject(InvitationApiService);
  protected readonly auth = inject(AuthService);
  protected readonly bootstrapApi = inject(BootstrapApiService);

  private querySub: Subscription | null = null;

  protected readonly phase = signal<UiPhase>('loading_preview');
  protected readonly token = signal<string>('');
  protected readonly preview = signal<InvitationPreview | null>(null);
  protected readonly previewError = signal<string | null>(null);
  protected readonly serverError = signal<string | null>(null);
  protected readonly submitting = signal(false);

  protected readonly registerForm = this.fb.nonNullable.group(
    {
      fullname: ['', [Validators.required, Validators.maxLength(512)]],
      username: ['', [Validators.required, Validators.pattern(USERNAME_PATTERN)]],
      password: ['', [Validators.required, Validators.minLength(8), phase0PasswordRules]],
      passwordConfirmation: ['', Validators.required],
    },
    { validators: passwordConfirmationMatch },
  );

  ngOnInit(): void {
    this.bootstrapApi.loadBootstrap();
    this.querySub = this.route.queryParamMap.subscribe((map) => {
      const id = map.get('id')?.trim() ?? '';
      this.token.set(id);
      this.preview.set(null);
      this.previewError.set(null);
      this.serverError.set(null);
      if (!id) {
        this.phase.set('missing_token');
        return;
      }
      this.phase.set('loading_preview');
      this.invitationApi.previewInvitation(id).subscribe({
        next: (p) => {
          this.preview.set(p);
          this.afterPreviewLoaded(p);
        },
        error: (err: unknown) => {
          this.previewError.set(InvitationApiService.errorMessage(err));
          this.phase.set('preview_error');
        },
      });
    });
  }

  ngOnDestroy(): void {
    this.querySub?.unsubscribe();
  }

  private afterPreviewLoaded(p: InvitationPreview): void {
    if (p.status !== 'pending') {
      this.previewError.set('This invitation is no longer valid.');
      this.phase.set('preview_error');
      return;
    }
    if (!p.existingUserPresent) {
      this.phase.set('register');
      return;
    }
    this.phase.set('loading_preview');
    this.auth.ensureAuthenticated().subscribe((ok) => {
      if (!ok) {
        this.phase.set('login_required');
        return;
      }
      const u = this.auth.currentUser();
      if (!u) {
        this.phase.set('login_required');
        return;
      }
      if (u.email.toLowerCase() !== p.invitedEmail.toLowerCase()) {
        this.phase.set('wrong_account');
        return;
      }
      this.phase.set('confirm');
    });
  }

  protected roleLabel(role: CompanyRole): string {
    const labels: Record<CompanyRole, string> = {
      company_admin: 'Company Admin',
      project_manager: 'Project Manager',
      contributor: 'Contributor',
      viewer: 'Viewer',
    };
    return labels[role];
  }

  protected loginReturnTo(): string {
    const t = this.token();
    return `/invitations/accept?id=${encodeURIComponent(t)}`;
  }

  protected submitRegister(): void {
    this.serverError.set(null);
    this.registerForm.markAllAsTouched();
    if (this.registerForm.invalid || this.submitting()) {
      return;
    }
    const t = this.token();
    if (!t) {
      return;
    }
    const { fullname, username, password, passwordConfirmation } =
      this.registerForm.getRawValue();
    this.submitting.set(true);
    this.invitationApi
      .acceptInvitationRegister({
        token: t,
        fullname,
        username,
        password,
        passwordConfirmation,
      })
      .subscribe({
        next: (user) => {
          this.submitting.set(false);
          this.auth.setSessionUser(user);
          void this.router.navigateByUrl('/app/projects');
        },
        error: (err: unknown) => {
          this.submitting.set(false);
          this.serverError.set(InvitationApiService.errorMessage(err));
        },
      });
  }

  protected confirmAccept(): void {
    const t = this.token();
    if (!t || this.submitting()) {
      return;
    }
    this.serverError.set(null);
    this.submitting.set(true);
    this.invitationApi.acceptInvitationConfirm(t).subscribe({
      next: (user) => {
        this.submitting.set(false);
        this.auth.setSessionUser(user);
        void this.router.navigateByUrl('/app/projects');
      },
      error: (err: unknown) => {
        this.submitting.set(false);
        this.serverError.set(InvitationApiService.errorMessage(err));
      },
    });
  }

  protected registerFieldError(
    controlName: 'fullname' | 'username' | 'password' | 'passwordConfirmation',
  ): string {
    const c = this.registerForm.controls[controlName];
    if (!c.touched || !c.errors) {
      return '';
    }
    if (c.errors['required']) {
      return 'This field is required.';
    }
    if (controlName === 'username' && c.errors['pattern']) {
      return 'Username must be 3–64 characters and may only contain letters, numbers, underscores, dots, and hyphens.';
    }
    if (controlName === 'password') {
      if (c.errors['minlength']) {
        return 'Password must be at least 8 characters.';
      }
      if (c.errors['phase0Password']) {
        return 'Password must include at least one letter and one number.';
      }
    }
    return '';
  }

  protected registerPasswordMismatchError(): string {
    const g = this.registerForm;
    if (!g.touched || !g.errors?.['passwordMismatch']) {
      return '';
    }
    return 'Password and confirmation do not match.';
  }

  protected logoutAndSignIn(): void {
    this.auth.logout().subscribe({
      next: () => {
        void this.router.navigate(['/login'], {
          queryParams: { returnTo: this.loginReturnTo() },
        });
      },
    });
  }
}
