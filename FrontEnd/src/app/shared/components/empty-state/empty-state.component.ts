import { Component, input, CUSTOM_ELEMENTS_SCHEMA } from '@angular/core';

@Component({
  selector: 'app-empty-state',
  standalone: true,
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  template: `
    <div class="flex flex-col items-center justify-center py-12 text-center">
      <iconify-icon [icon]="icon()" width="48" class="text-neutral-700 mb-4"></iconify-icon>
      <p class="text-sm text-neutral-500 mb-1">{{ title() }}</p>
      @if (subtitle()) {
        <p class="text-xs text-neutral-600">{{ subtitle() }}</p>
      }
    </div>
  `,
})
export class EmptyStateComponent {
  icon = input<string>('mdi:inbox-outline');
  title = input<string>('Aucune donnée');
  subtitle = input<string>('');
}
