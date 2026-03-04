import { Injectable, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Router } from '@angular/router';
import { Observable, tap } from 'rxjs';
import { environment } from '../../../environments/environment';
import {
  User,
  LoginRequest,
  RegisterRequest,
  AuthResponse,
  GoogleAuthRequest,
  ChangePasswordRequest,
  ForgotPasswordRequest,
  ResetPasswordRequest,
  ApiMessage,
} from '../models/auth.model';

const TOKEN_KEY = 'neural_token';
const USER_KEY = 'neural_user';

@Injectable({ providedIn: 'root' })
export class AuthService {
  private readonly api = environment.apiBaseUrl;
  private readonly currentUser = signal<User | null>(this.loadUser());
  private readonly token = signal<string | null>(this.loadToken());

  readonly user = this.currentUser.asReadonly();
  readonly isAuthenticated = computed(() => !!this.token());

  constructor(
    private readonly http: HttpClient,
    private readonly router: Router,
  ) {}

  login(req: LoginRequest): Observable<AuthResponse> {
    return this.http.post<AuthResponse>(`${this.api}/auth/login`, req).pipe(
      tap((res) => this.handleAuth(res)),
    );
  }

  register(req: RegisterRequest): Observable<AuthResponse> {
    return this.http.post<AuthResponse>(`${this.api}/auth/register`, req).pipe(
      tap((res) => this.handleAuth(res)),
    );
  }

  googleAuth(req: GoogleAuthRequest): Observable<AuthResponse> {
    return this.http.post<AuthResponse>(`${this.api}/auth/google`, req).pipe(
      tap((res) => this.handleAuth(res)),
    );
  }

  changePassword(req: ChangePasswordRequest): Observable<ApiMessage> {
    return this.http.post<ApiMessage>(`${this.api}/auth/change-password`, req);
  }

  forgotPassword(req: ForgotPasswordRequest): Observable<ApiMessage> {
    return this.http.post<ApiMessage>(`${this.api}/auth/forgot-password`, req);
  }

  resetPassword(req: ResetPasswordRequest): Observable<ApiMessage> {
    return this.http.post<ApiMessage>(`${this.api}/auth/reset-password`, req);
  }

  verifyEmail(token: string): Observable<ApiMessage> {
    return this.http.get<ApiMessage>(`${this.api}/auth/verify-email`, {
      params: { token },
    });
  }

  me(): Observable<User> {
    return this.http.get<User>(`${this.api}/auth/me`).pipe(
      tap((user) => {
        this.currentUser.set(user);
        localStorage.setItem(USER_KEY, JSON.stringify(user));
      }),
    );
  }

  getToken(): string | null {
    return this.token();
  }

  logout(): void {
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(USER_KEY);
    this.token.set(null);
    this.currentUser.set(null);
    this.router.navigate(['/login']);
  }

  private handleAuth(res: AuthResponse): void {
    localStorage.setItem(TOKEN_KEY, res.token);
    localStorage.setItem(USER_KEY, JSON.stringify(res.user));
    this.token.set(res.token);
    this.currentUser.set(res.user);
  }

  private loadToken(): string | null {
    return localStorage.getItem(TOKEN_KEY);
  }

  private loadUser(): User | null {
    const raw = localStorage.getItem(USER_KEY);
    if (!raw) return null;
    try {
      return JSON.parse(raw) as User;
    } catch {
      return null;
    }
  }
}
