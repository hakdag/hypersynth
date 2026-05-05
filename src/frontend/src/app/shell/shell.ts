import { Component, OnInit, inject } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';
import { BootstrapApiService } from '../bootstrap-api.service';

@Component({
  selector: 'app-shell',
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './shell.html',
  styleUrl: './shell.scss',
})
export class Shell implements OnInit {
  protected readonly bootstrapApi = inject(BootstrapApiService);

  ngOnInit(): void {
    this.bootstrapApi.loadBootstrap();
  }
}
