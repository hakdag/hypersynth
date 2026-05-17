import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-system-admin-dashboard',
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './system-admin-dashboard.html',
  styleUrl: './system-admin-dashboard.scss',
})
export class SystemAdminDashboard {}
