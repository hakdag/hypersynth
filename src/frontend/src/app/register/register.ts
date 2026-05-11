import { CommonModule } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import {
  AbstractControl,
  FormBuilder,
  ReactiveFormsModule,
  ValidationErrors,
  Validators,
} from '@angular/forms';
import { RouterLink } from '@angular/router';

import { BootstrapApiService } from '../bootstrap-api.service';
import {
  RegistrationApiService,
  RegisterAccountType,
} from '../registration-api.service';

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

@Component({
  selector: 'app-register',
  imports: [CommonModule, ReactiveFormsModule, RouterLink],
  templateUrl: './register.html',
  styleUrl: './register.scss',
})
export class Register implements OnInit {
  private readonly fb = inject(FormBuilder);
  private readonly registrationApi = inject(RegistrationApiService);
  protected readonly bootstrapApi = inject(BootstrapApiService);

  protected readonly submitting = signal(false);
  protected readonly serverError = signal<string | null>(null);
  protected readonly serverSuccess = signal<string | null>(null);
  protected readonly selectedAccountType = signal<RegisterAccountType | null>(null);

  protected readonly form = this.fb.nonNullable.group({
    fullname: ['', [Validators.required, Validators.maxLength(512)]],
    email: ['', [Validators.required, Validators.email]],
    password: ['', [Validators.required, Validators.minLength(8), phase0PasswordRules]],
    terms: [false, Validators.requiredTrue],
  });

  ngOnInit(): void {
    this.bootstrapApi.loadBootstrap();
  }

  protected selectAccountType(accountType: RegisterAccountType): void {
    this.serverError.set(null);
    this.serverSuccess.set(null);
    this.selectedAccountType.set(accountType);
  }

  protected changeAccountType(): void {
    this.serverError.set(null);
    this.serverSuccess.set(null);
    this.selectedAccountType.set(null);
  }

  protected accountTypeLabel(): string {
    const accountType = this.selectedAccountType();
    if (accountType === 'company') {
      return 'Company Account';
    }
    if (accountType === 'personal') {
      return 'Personal Account';
    }
    return '';
  }

  protected submit(): void {
    this.serverError.set(null);
    this.serverSuccess.set(null);
    this.form.markAllAsTouched();
    const accountType = this.selectedAccountType();
    if (this.form.invalid || this.submitting() || accountType === null) {
      return;
    }

    const { fullname, email, password } = this.form.getRawValue();
    this.submitting.set(true);

    this.registrationApi.register({ accountType, fullname, email, password }).subscribe({
      next: (res) => {
        this.submitting.set(false);
        this.serverSuccess.set(res.message);
        this.form.reset({ terms: false });
      },
      error: (err: unknown) => {
        this.submitting.set(false);
        this.serverError.set(RegistrationApiService.errorMessage(err));
      },
    });
  }

  protected fieldError(controlName: 'fullname' | 'email' | 'password' | 'terms'): string {
    const c = this.form.controls[controlName];
    if (!c.touched || !c.errors) {
      return '';
    }
    if (c.errors['required']) {
      if (controlName === 'terms') {
        return 'You must accept the terms to continue.';
      }
      return 'This field is required.';
    }
    if (controlName === 'email' && c.errors['email']) {
      return 'Enter a valid email address.';
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
}
