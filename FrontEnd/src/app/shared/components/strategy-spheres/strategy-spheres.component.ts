import { Component, input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Strategy } from '../../../core/models/trade.model';

@Component({
  selector: 'app-strategy-spheres',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './strategy-spheres.component.html',
})
export class StrategySpheresComponent {
  strategies = input.required<Strategy[]>();

  getSphereClass(recommendation: string): string {
    switch (recommendation) {
      case 'BUY': return 'strategy-sphere buy';
      case 'SELL': return 'strategy-sphere sell';
      default: return 'strategy-sphere hold';
    }
  }

  getSphereLabel(recommendation: string): string {
    switch (recommendation) {
      case 'BUY': return 'B';
      case 'SELL': return 'S';
      default: return 'H';
    }
  }
}
