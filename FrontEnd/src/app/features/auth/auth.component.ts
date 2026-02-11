import { Component, signal, inject, CUSTOM_ELEMENTS_SCHEMA } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { AuthService } from '../../core/services/auth.service';

@Component({
  selector: 'app-auth',
  standalone: true,
  imports: [CommonModule, FormsModule],
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  templateUrl: './auth.component.html',
})
export class AuthComponent {
  private readonly authService = inject(AuthService);
  private readonly router = inject(Router);

  // Login form
  loginEmail = signal('');
  loginPassword = signal('');
  loginShowPassword = signal(false);
  loginLoading = signal(false);
  loginError = signal('');

  // Signup form
  signupName = signal('');
  signupEmail = signal('');
  signupPassword = signal('');
  signupShowPassword = signal(false);
  signupTerms = signal(false);
  signupLoading = signal(false);
  signupError = signal('');
  signupSuccess = signal('');

  onLogin(): void {
    this.loginError.set('');
    this.loginLoading.set(true);
    this.authService
      .login({
        email: this.loginEmail(),
        password: this.loginPassword(),
      })
      .subscribe({
        next: () => {
          this.loginLoading.set(false);
          this.router.navigate(['/dashboard']);
        },
        error: (err) => {
          this.loginLoading.set(false);
          this.loginError.set(err.error?.error || 'Login failed');
        },
      });
  }

  onSignup(): void {
    if (!this.signupTerms()) return;
    this.signupError.set('');
    this.signupSuccess.set('');
    this.signupLoading.set(true);
    this.authService
      .register({
        username: this.signupName(),
        email: this.signupEmail(),
        password: this.signupPassword(),
      })
      .subscribe({
        next: (res) => {
          this.signupLoading.set(false);
          this.signupSuccess.set(res.message || 'Account created!');
          this.router.navigate(['/dashboard']);
        },
        error: (err) => {
          this.signupLoading.set(false);
          this.signupError.set(err.error?.error || 'Registration failed');
        },
      });
  }

  toggleLoginPassword(): void {
    this.loginShowPassword.update((v) => !v);
  }

  toggleSignupPassword(): void {
    this.signupShowPassword.update((v) => !v);
  }

  private readonly particleConfig: Record<number, { left: string; dur: string; delay: string }> = {
    1: { left: '10%', dur: '25s', delay: '0s' },
    2: { left: '20%', dur: '30s', delay: '2s' },
    3: { left: '30%', dur: '28s', delay: '4s' },
    4: { left: '40%', dur: '32s', delay: '1s' },
    5: { left: '50%', dur: '27s', delay: '3s' },
    6: { left: '60%', dur: '29s', delay: '5s' },
    7: { left: '70%', dur: '31s', delay: '0s' },
    8: { left: '80%', dur: '26s', delay: '2s' },
    9: { left: '90%', dur: '33s', delay: '4s' },
    10: { left: '15%', dur: '28s', delay: '1s' },
    11: { left: '25%', dur: '30s', delay: '3s' },
    12: { left: '35%', dur: '27s', delay: '5s' },
    13: { left: '45%', dur: '31s', delay: '0s' },
    14: { left: '55%', dur: '29s', delay: '2s' },
    15: { left: '65%', dur: '32s', delay: '4s' },
    16: { left: '75%', dur: '26s', delay: '1s' },
    17: { left: '85%', dur: '28s', delay: '3s' },
    18: { left: '95%', dur: '30s', delay: '5s' },
    19: { left: '5%', dur: '27s', delay: '2s' },
    20: { left: '12%', dur: '29s', delay: '4s' },
  };

  getParticleLeft(n: number): string { return this.particleConfig[n]?.left ?? '0%'; }
  getParticleDuration(n: number): string { return this.particleConfig[n]?.dur ?? '25s'; }
  getParticleDelay(n: number): string { return this.particleConfig[n]?.delay ?? '0s'; }
}
