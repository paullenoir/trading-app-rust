import { Routes } from '@angular/router';
import { authGuard, guestGuard } from './core/guards/auth.guard';

export const routes: Routes = [
  {
    path: 'login',
    canActivate: [guestGuard],
    loadComponent: () =>
      import('./features/auth/auth.component').then((m) => m.AuthComponent),
  },
  {
    path: 'dashboard',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/dashboard/dashboard.component').then(
        (m) => m.DashboardComponent,
      ),
  },
  {
    path: 'wallet',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/wallet/wallet.component').then(
        (m) => m.WalletComponent,
      ),
  },
  {
    path: 'trade',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/trade/trade.component').then(
        (m) => m.TradeComponent,
      ),
  },
  {
    path: 'chart',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/chart/chart.component').then(
        (m) => m.ChartComponent,
      ),
  },
  {
    path: 'chart/:symbol',
    canActivate: [authGuard],
    loadComponent: () =>
      import('./features/chart/chart.component').then(
        (m) => m.ChartComponent,
      ),
  },
  { path: '', redirectTo: 'dashboard', pathMatch: 'full' },
  { path: '**', redirectTo: 'dashboard' },
];
