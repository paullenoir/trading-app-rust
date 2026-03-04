import { Component, input } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-loading-skeleton',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="animate-pulse space-y-4">
      @for (i of rows(); track i) {
        <div class="dashboard-card p-5">
          <div class="flex items-center gap-3 mb-3">
            <div class="w-8 h-8 bg-white/5 rounded-lg"></div>
            <div class="h-4 bg-white/5 rounded w-32"></div>
          </div>
          <div class="h-8 bg-white/5 rounded w-48"></div>
        </div>
      }
    </div>
  `,
})
export class LoadingSkeletonComponent {
  count = input<number>(3);
  rows = input<number[]>([1, 2, 3]);
}
