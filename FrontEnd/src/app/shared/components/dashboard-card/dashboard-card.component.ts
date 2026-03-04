import { Component, input } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-dashboard-card',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="dashboard-card p-5">
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center gap-3">
          <div class="w-1 h-6 rounded-full" [class]="barColor()"></div>
          <div>
            <h2 class="text-base font-bold text-white font-geist">{{ title() }}</h2>
            @if (subtitle()) {
              <div class="flex items-center gap-3 text-xs mt-1">
                <ng-content select="[subtitle]"></ng-content>
              </div>
            }
          </div>
        </div>
        <ng-content select="[header-actions]"></ng-content>
      </div>
      <ng-content></ng-content>
    </div>
  `,
})
export class DashboardCardComponent {
  title = input.required<string>();
  barColor = input<string>('bg-emerald-500');
  subtitle = input<boolean>(false);
}
