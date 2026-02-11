import { Component, input, output } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-filter-buttons',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="flex items-center gap-2 flex-wrap">
      @for (opt of options(); track opt) {
        <button (click)="filterChange.emit(opt)"
          class="px-3 py-1.5 text-xs font-medium rounded-lg transition-colors"
          [class]="opt === selected() ? activeClass() : 'text-neutral-500 hover:text-white hover:bg-white/5 border border-white/10'">
          {{ opt }}
        </button>
      }
    </div>
  `,
})
export class FilterButtonsComponent {
  options = input.required<string[]>();
  selected = input.required<string>();
  activeClass = input<string>('text-white bg-emerald-500 rounded shadow-[0_0_10px_rgba(16,185,129,0.3)]');
  filterChange = output<string>();
}
