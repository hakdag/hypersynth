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
import { buildTimezoneOptions, COUNTRY_OPTIONS } from '../company-form-options';
import {
  RegistrationApiService,
  RegisterAccountType,
} from '../registration-api.service';

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
  protected readonly countryOptions = COUNTRY_OPTIONS;
  protected readonly timezoneOptions = buildTimezoneOptions();

  protected readonly personalForm = this.fb.nonNullable.group({
    fullname: ['', [Validators.required, Validators.maxLength(512)]],
    email: ['', [Validators.required, Validators.email]],
    password: ['', [Validators.required, Validators.minLength(8), phase0PasswordRules]],
    terms: [false, Validators.requiredTrue],
  });

  protected readonly companyForm = this.fb.nonNullable.group({
    company: this.fb.nonNullable.group({
      name: ['', [Validators.required, Validators.maxLength(255)]],
      companyEmail: ['', [Validators.required, Validators.email]],
      country: ['', Validators.required],
      timezone: ['', Validators.required],
    }),
    admin: this.fb.nonNullable.group(
      {
        fullName: ['', [Validators.required, Validators.maxLength(512)]],
        email: ['', [Validators.required, Validators.email]],
        username: ['', [Validators.required, Validators.pattern(USERNAME_PATTERN)]],
        password: ['', [Validators.required, Validators.minLength(8), phase0PasswordRules]],
        passwordConfirmation: ['', Validators.required],
      },
      { validators: passwordConfirmationMatch },
    ),
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
    this.personalForm.reset({ terms: false });
    this.companyForm.reset({
      company: { name: '', companyEmail: '', country: '', timezone: '' },
      admin: {
        fullName: '',
        email: '',
        username: '',
        password: '',
        passwordConfirmation: '',
      },
      terms: false,
    });
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

  protected submitPersonal(): void {
    this.serverError.set(null);
    this.serverSuccess.set(null);
    this.personalForm.markAllAsTouched();
    if (this.personalForm.invalid || this.submitting()) {
      return;
    }

    const { fullname, email, password } = this.personalForm.getRawValue();
    this.submitting.set(true);

    this.registrationApi.register({ accountType: 'personal', fullname, email, password }).subscribe({
      next: (res) => {
        this.submitting.set(false);
        this.serverSuccess.set(res.message);
        this.personalForm.reset({ terms: false });
      },
      error: (err: unknown) => {
        this.submitting.set(false);
        this.serverError.set(RegistrationApiService.errorMessage(err));
      },
    });
  }

  protected submitCompany(): void {
    this.serverError.set(null);
    this.serverSuccess.set(null);
    this.companyForm.markAllAsTouched();
    if (this.companyForm.invalid || this.submitting()) {
      return;
    }

    const { company, admin } = this.companyForm.getRawValue();
    this.submitting.set(true);

    this.registrationApi
      .registerCompany({
        name: company.name,
        companyEmail: company.companyEmail,
        country: company.country,
        timezone: company.timezone,
        fullName: admin.fullName,
        email: admin.email,
        username: admin.username,
        password: admin.password,
        passwordConfirmation: admin.passwordConfirmation,
      })
      .subscribe({
        next: (res) => {
          this.submitting.set(false);
          this.serverSuccess.set(res.message);
          this.companyForm.reset({
            company: { name: '', companyEmail: '', country: '', timezone: '' },
            admin: {
              fullName: '',
              email: '',
              username: '',
              password: '',
              passwordConfirmation: '',
            },
            terms: false,
          });
        },
        error: (err: unknown) => {
          this.submitting.set(false);
          this.serverError.set(RegistrationApiService.errorMessage(err));
        },
      });
  }

  protected personalFieldError(
    controlName: 'fullname' | 'email' | 'password' | 'terms',
  ): string {
    const c = this.personalForm.controls[controlName];
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

  protected companyDetailsFieldError(
    controlName: 'name' | 'companyEmail' | 'country' | 'timezone',
  ): string {
    const c = this.companyForm.controls.company.controls[controlName];
    return this.controlErrorMessage(c);
  }

  protected companyAdminFieldError(
    controlName: 'fullName' | 'email' | 'username' | 'password' | 'passwordConfirmation',
  ): string {
    const c = this.companyForm.controls.admin.controls[controlName];
    return this.controlErrorMessage(c);
  }

  private controlErrorMessage(c: AbstractControl): string {
    if (!c.touched || !c.errors) {
      return '';
    }
    if (c.errors['required']) {
      return 'This field is required.';
    }
    if (c.errors['email']) {
      return 'Enter a valid email address.';
    }
    if (c.errors['maxlength']) {
      return 'This value is too long.';
    }
    if (c.errors['pattern']) {
      return 'Username must be 3–64 characters and may only contain letters, numbers, underscores, dots, and hyphens.';
    }
    if (c.errors['minlength']) {
      return 'Password must be at least 8 characters.';
    }
    if (c.errors['phase0Password']) {
      return 'Password must include at least one letter and one number.';
    }
    return '';
  }

  protected companyTermsError(): string {
    const c = this.companyForm.controls.terms;
    if (!c.touched || !c.errors) {
      return '';
    }
    if (c.errors['required']) {
      return 'You must accept the terms to continue.';
    }
    return '';
  }

  protected companyPasswordMismatchError(): string {
    const admin = this.companyForm.controls.admin;
    if (!admin.touched || !admin.errors?.['passwordMismatch']) {
      return '';
    }
    return 'Password and confirmation do not match.';
  }
}
