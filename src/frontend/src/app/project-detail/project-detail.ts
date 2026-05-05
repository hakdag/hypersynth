import { Component, computed, inject } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { map } from 'rxjs';
import { BootstrapApiService } from '../bootstrap-api.service';

@Component({
  selector: 'app-project-detail',
  imports: [RouterLink],
  templateUrl: './project-detail.html',
  styleUrl: './project-detail.scss',
})
export class ProjectDetail {
  private readonly route = inject(ActivatedRoute);
  protected readonly bootstrapApi = inject(BootstrapApiService);

  readonly projectId = toSignal(this.route.paramMap.pipe(map((p) => p.get('projectId') ?? '')), {
    initialValue: '',
  });

  readonly subtitle = computed(() => {
    const id = this.projectId();
    return id.length > 0
      ? `Placeholder detail for "${id}". No real project records exist yet.`
      : 'No project identifier provided.';
  });
}
