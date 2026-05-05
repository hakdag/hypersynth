import { Component, inject } from '@angular/core';
import { RouterLink } from '@angular/router';
import { BootstrapApiService } from '../bootstrap-api.service';

@Component({
  selector: 'app-project-list',
  imports: [RouterLink],
  templateUrl: './project-list.html',
  styleUrl: './project-list.scss',
})
export class ProjectList {
  protected readonly bootstrapApi = inject(BootstrapApiService);
}
