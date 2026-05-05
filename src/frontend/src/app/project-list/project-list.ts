import { Component, inject, OnInit, signal } from '@angular/core';
import { RouterLink } from '@angular/router';

import { BootstrapApiService } from '../bootstrap-api.service';
import { CreatedProject, ProjectApiService } from '../project-api.service';

@Component({
  selector: 'app-project-list',
  imports: [RouterLink],
  templateUrl: './project-list.html',
  styleUrl: './project-list.scss',
})
export class ProjectList implements OnInit {
  protected readonly bootstrapApi = inject(BootstrapApiService);
  private readonly projectApi = inject(ProjectApiService);

  protected readonly projects = signal<CreatedProject[]>([]);
  protected readonly loadState = signal<'idle' | 'loading' | 'ok' | 'error'>('loading');
  protected readonly listError = signal<string | null>(null);

  ngOnInit(): void {
    this.loadProjects();
  }

  protected loadProjects(): void {
    this.listError.set(null);
    this.loadState.set('loading');
    this.projectApi.listProjects().subscribe({
      next: (rows) => {
        this.projects.set(rows);
        this.loadState.set('ok');
      },
      error: (err: unknown) => {
        this.loadState.set('error');
        this.listError.set(ProjectApiService.listErrorMessage(err));
      },
    });
  }
}
