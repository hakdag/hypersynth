import { CommonModule } from '@angular/common';
import { Component, OnInit, inject, signal } from '@angular/core';
import { AbstractControl, FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';

import { AuthService } from '../auth.service';
import { CompanyApiService, Company } from '../company-api.service';
import { buildTimezoneOptions, COUNTRY_OPTIONS } from '../company-form-options';

type LoadState = 'loading' | 'ok' | 'error';

@Component({
  selector: 'app-company-profile',
  imports: [CommonModule, ReactiveFormsModule],
  templateUrl: './company-profile.html',
  styleUrl: './company-profile.scss',
})
export class CompanyProfile implements OnInit {
  private readonly fb = inject(FormBuilder);
  private readonly companyApi = inject(CompanyApiService);
  private readonly auth = inject(AuthService);

  protected readonly canManageCompanyProfile = this.auth.canManageCompanyProfile;

  protected readonly loadState = signal<LoadState>('loading');
  protected readonly loadError = signal<string | null>(null);
  protected readonly submitting = signal(false);
  protected readonly serverError = signal<string | null>(null);
  protected readonly serverSuccess = signal<string | null>(null);
  protected readonly updatedAt = signal<string | null>(null);
  protected readonly countryOptions = COUNTRY_OPTIONS;
  protected readonly timezoneOptions = buildTimezoneOptions();

  protected readonly form = this.fb.nonNullable.group({
    name: ['', [Validators.required, Validators.maxLength(255)]],
    companyEmail: ['', [Validators.required, Validators.email]],
    country: ['', Validators.required],
    timezone: ['', Validators.required],
    legalName: [''],
    website: [''],
    industry: [''],
    companySize: [''],
    phone: [''],
    billingEmail: ['', Validators.email],
    address: [''],
    taxVatNumber: [''],
  });

  ngOnInit(): void {
    if (!this.canManageCompanyProfile()) {
      this.form.disable({ emitEvent: false });
    }
    this.companyApi.getCompany().subscribe({
      next: (company) => {
        this.patchForm(company);
        this.updatedAt.set(company.updatedAt);
        this.loadState.set('ok');
      },
      error: (err: unknown) => {
        this.loadError.set(CompanyApiService.errorMessage(err));
        this.loadState.set('error');
      },
    });
  }

  protected submit(): void {
    this.serverError.set(null);
    this.serverSuccess.set(null);
    this.form.markAllAsTouched();
    if (this.form.invalid || this.submitting()) {
      return;
    }

    const raw = this.form.getRawValue();
    this.submitting.set(true);

    this.companyApi
      .updateCompany({
        name: raw.name,
        companyEmail: raw.companyEmail,
        country: raw.country,
        timezone: raw.timezone,
        legalName: this.optionalPayload(raw.legalName),
        website: this.optionalPayload(raw.website),
        industry: this.optionalPayload(raw.industry),
        companySize: this.optionalPayload(raw.companySize),
        phone: this.optionalPayload(raw.phone),
        billingEmail: this.optionalPayload(raw.billingEmail),
        address: this.optionalPayload(raw.address),
        taxVatNumber: this.optionalPayload(raw.taxVatNumber),
      })
      .subscribe({
        next: (company) => {
          this.submitting.set(false);
          this.patchForm(company);
          this.updatedAt.set(company.updatedAt);
          this.serverSuccess.set('Company profile saved.');
        },
        error: (err: unknown) => {
          this.submitting.set(false);
          this.serverError.set(CompanyApiService.errorMessage(err));
        },
      });
  }

  protected fieldError(
    controlName:
      | 'name'
      | 'companyEmail'
      | 'country'
      | 'timezone'
      | 'legalName'
      | 'website'
      | 'industry'
      | 'companySize'
      | 'phone'
      | 'billingEmail'
      | 'address'
      | 'taxVatNumber',
  ): string {
    const c = this.form.controls[controlName];
    return this.controlErrorMessage(c);
  }

  protected formatUpdatedAt(value: string | null): string {
    if (value === null || value.length === 0) {
      return '';
    }
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
      return value;
    }
    return date.toLocaleString();
  }

  private patchForm(company: Company): void {
    this.form.patchValue({
      name: company.name,
      companyEmail: company.companyEmail,
      country: company.country,
      timezone: company.timezone,
      legalName: company.legalName ?? '',
      website: company.website ?? '',
      industry: company.industry ?? '',
      companySize: company.companySize ?? '',
      phone: company.phone ?? '',
      billingEmail: company.billingEmail ?? '',
      address: company.address ?? '',
      taxVatNumber: company.taxVatNumber ?? '',
    });
  }

  private optionalPayload(value: string): string | null {
    const trimmed = value.trim();
    return trimmed.length === 0 ? null : trimmed;
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
    return '';
  }
}
