import { Component, OnInit, inject } from '@angular/core';
import { Router, RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

import { AuthService } from '../auth.service';
import { BootstrapApiService } from '../bootstrap-api.service';
import { CompanyAccessService } from '../company-access.service';

@Component({
  selector: 'app-shell',
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './shell.html',
  styleUrl: './shell.scss',
})
export class Shell implements OnInit {
  protected readonly bootstrapApi = inject(BootstrapApiService);
  private readonly auth = inject(AuthService);
  private readonly companyAccess = inject(CompanyAccessService);
  private readonly router = inject(Router);

  readonly currentUser = this.auth.currentUser;
  readonly hasCompanyAssociation = this.companyAccess.hasCompanyAssociation;

  ngOnInit(): void {
    this.bootstrapApi.loadBootstrap();
    this.companyAccess.resolveHasCompanyAssociation().subscribe();
  }

  /** Dashboard entry is reserved for a future home route; project list lives under `/app/projects`. */
  protected dashboardNavActive(): boolean {
    const path = this.router.url.split('?')[0];
    return path === '/app' || path === '/app/';
  }

  protected projectsNavActive(): boolean {
    const path = this.router.url.split('?')[0];
    return path === '/app/projects' || path.startsWith('/app/projects/');
  }

  protected companyNavActive(): boolean {
    const path = this.router.url.split('?')[0];
    return path === '/app/company' || path.startsWith('/app/company/');
  }

  protected logout(): void {
    this.auth.logout().subscribe({
      next: () => void this.router.navigate(['/login']),
      error: () => void this.router.navigate(['/login']),
    });
  }
}
