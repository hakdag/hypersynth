import { CommonModule } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';

import { AuthApiService } from '../auth-api.service';
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

  protected readonly form = this.fb.nonNullable.group({
    email: ['', [Validators.required, Validators.email]],
    password: ['', [Validators.required]],
  });

  ngOnInit(): void {
    this.bootstrapApi.loadBootstrap();

    this.auth.ensureAuthenticated().subscribe((ok) => {
      if (ok) {
        const target = this.route.snapshot.queryParamMap.get('returnUrl') || '/app/projects';
        void this.router.navigateByUrl(target);
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
      next: () => {
        this.submitting.set(false);
        const target = this.route.snapshot.queryParamMap.get('returnUrl') || '/app/projects';
        void this.router.navigateByUrl(target);
      },
      error: (err: unknown) => {
        this.submitting.set(false);
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
