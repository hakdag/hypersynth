import { CommonModule } from '@angular/common';
import { Component, inject, input, OnInit, output, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { finalize } from 'rxjs';

import { DocumentContextPickerModal } from '../document-context-picker-modal/document-context-picker-modal';
import {
  CreatedTask,
  GeneratedTaskCandidate,
  ProjectApiService,
  TaskGenerationTurn,
} from '../project-api.service';

@Component({
  selector: 'app-task-ai-generate-dialog',
  imports: [CommonModule, FormsModule, DocumentContextPickerModal],
  templateUrl: './task-ai-generate-dialog.html',
  styleUrl: './task-ai-generate-dialog.scss',
})
export class TaskAiGenerateDialog implements OnInit {
  private readonly projectApi = inject(ProjectApiService);

  readonly projectId = input.required<string>();
  readonly featureId = input.required<string>();
  readonly projectName = input<string>('');

  readonly closed = output<void>();
  readonly accepted = output<CreatedTask[]>();

  protected readonly generating = signal(false);
  protected readonly accepting = signal(false);
  protected readonly error = signal<string | null>(null);
  protected readonly proposed = signal<GeneratedTaskCandidate[]>([]);
  protected readonly history = signal<TaskGenerationTurn[]>([]);
  protected readonly documentContextPickerOpen = signal(false);
  protected readonly aiSelectedDocumentIds = signal<string[]>([]);
  protected feedbackDraft = '';

  ngOnInit(): void {
    this.runGenerate([]);
  }

  protected cancel(): void {
    this.closed.emit();
  }

  protected openDocumentContextPicker(): void {
    if (this.generating() || this.accepting()) {
      return;
    }
    this.documentContextPickerOpen.set(true);
  }

  protected onDocumentContextConfirmed(ids: string[]): void {
    this.aiSelectedDocumentIds.set(ids);
    this.documentContextPickerOpen.set(false);
  }

  protected closeDocumentContextPicker(): void {
    this.documentContextPickerOpen.set(false);
  }

  protected acceptAll(): void {
    const tasks = this.proposed();
    if (tasks.length === 0 || this.accepting() || this.generating()) {
      return;
    }
    this.accepting.set(true);
    this.error.set(null);
    this.projectApi
      .acceptGeneratedTasks(this.projectId(), this.featureId(), tasks)
      .pipe(finalize(() => this.accepting.set(false)))
      .subscribe({
        next: (rows) => {
          this.accepted.emit(rows);
        },
        error: (err: unknown) => {
          this.error.set(ProjectApiService.acceptGeneratedTasksErrorMessage(err));
        },
      });
  }

  protected retryGeneration(): void {
    if (this.generating() || this.accepting()) {
      return;
    }
    this.runGenerate([...this.history()]);
  }

  protected regenerate(): void {
    const fb = this.feedbackDraft.trim();
    if (!fb || this.generating() || this.accepting()) {
      return;
    }
    const nextHistory: TaskGenerationTurn[] = [
      ...this.history(),
      { proposedTasks: [...this.proposed()], feedback: fb },
    ];
    this.history.set(nextHistory);
    this.feedbackDraft = '';
    this.runGenerate(nextHistory);
  }

  protected regenerateDisabled(): boolean {
    return (
      this.feedbackDraft.trim().length === 0 ||
      this.generating() ||
      this.accepting() ||
      this.proposed().length === 0
    );
  }

  private runGenerate(history: TaskGenerationTurn[]): void {
    this.generating.set(true);
    this.error.set(null);
    this.projectApi
      .generateFeatureTasks(
        this.projectId(),
        this.featureId(),
        history,
        this.aiSelectedDocumentIds(),
      )
      .pipe(finalize(() => this.generating.set(false)))
      .subscribe({
        next: (res) => {
          this.proposed.set(res.tasks);
        },
        error: (err: unknown) => {
          this.error.set(ProjectApiService.generateTasksErrorMessage(err));
        },
      });
  }
}
