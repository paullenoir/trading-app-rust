import { Component, input, CUSTOM_ELEMENTS_SCHEMA } from '@angular/core';

@Component({
  selector: 'app-kpi-card',
  standalone: true,
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  templateUrl: './kpi-card.component.html',
})
export class KpiCardComponent {
  icon = input.required<string>();
  iconColor = input<string>('text-emerald-500');
  iconBg = input<string>('bg-emerald-500/10');
  label = input.required<string>();
  value = input.required<string>();
  suffix = input<string>('');
  badge = input<string>('');
  badgeColor = input<string>('text-emerald-500 bg-emerald-500/10');
}
