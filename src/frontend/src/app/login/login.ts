import { CommonModule } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { ActivatedRoute, ParamMap, Router, RouterLink } from '@angular/router';

import { AuthApiService, CurrentUser } from '../auth-api.service';
import { AuthService } from '../auth.service';
import { BootstrapApiService } from '../bootstrap-api.service';

@Component({
  selector: 'app-login',
  imports: [CommonModule, ReactiveFormsModule, RouterLink],
  templateUrl: './login.html',
  styleUrl: './login.scss',
})
export class Login implements OnInit {
  private readonly fb = inject(FormBuilder);
  private readonly auth = inject(AuthService);
  private readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);
  protected readonly bootstrapApi = inject(BootstrapApiService);

  protected readonly submitting = signal(false);
  protected readonly serverError = signal<string | null>(null);
  protected readonly userDisabledNotice = signal(false);

  protected readonly form = this.fb.nonNullable.group({
    email: ['', [Validators.required, Validators.email]],
    password: ['', [Validators.required]],
  });

  ngOnInit(): void {
    this.bootstrapApi.loadBootstrap();

    if (this.route.snapshot.queryParamMap.get('reason') === 'user_disabled') {
      this.userDisabledNotice.set(true);
    }

    this.auth.ensureAuthenticated().subscribe((ok) => {
      if (ok) {
        const user = this.auth.currentUser();
        if (user) {
          void this.router.navigateByUrl(postLoginTarget(user, this.route.snapshot.queryParamMap));
        }
      }
    });
  }

  protected submit(): void {
    this.serverError.set(null);
    this.form.markAllAsTouched();
    if (this.form.invalid || this.submitting()) {
      return;
    }

    const { email, password } = this.form.getRawValue();
    this.submitting.set(true);

    this.auth.login(email, password).subscribe({
      next: (user) => {
        this.submitting.set(false);
        void this.router.navigateByUrl(postLoginTarget(user, this.route.snapshot.queryParamMap));
      },
      error: (err: unknown) => {
        this.submitting.set(false);
        if (AuthApiService.isCompanyDisabled(err)) {
          this.auth.clearSession();
          void this.router.navigateByUrl('/company-disabled');
          return;
        }
        if (AuthApiService.isUserDisabled(err)) {
          this.auth.clearSession();
          this.userDisabledNotice.set(true);
          this.serverError.set(null);
          return;
        }
        this.serverError.set(AuthApiService.loginErrorMessage(err));
      },
    });
  }

  protected fieldError(controlName: 'email' | 'password'): string {
    const c = this.form.controls[controlName];
    if (!c.touched || !c.errors) {
      return '';
    }
    if (c.errors['required']) {
      return 'This field is required.';
    }
    if (controlName === 'email' && c.errors['email']) {
      return 'Enter a valid email address.';
    }
    return '';
  }
}

function postLoginTarget(user: CurrentUser, map: ParamMap): string {
  if (user.accountType === 'system_admin') {
    return '/app/admin/companies';
  }
  return readReturnTarget(map);
}

function readReturnTarget(map: ParamMap): string {
  const raw = map.get('returnTo') ?? map.get('returnUrl');
  if (!raw || !raw.startsWith('/') || raw.startsWith('//')) {
    return '/app/projects';
  }
  return raw;
}
