import { Injectable } from '@angular/core';
import {
  OpenTradeWithRecommendations,
  ClosedTrade,
  ConsensusType,
  PositionWithGain,
  Strategy,
} from '../models/trade.model';
import { WalletHistoryItem } from '../models/wallet.model';

@Injectable({ providedIn: 'root' })
export class CalculationService {
  calculateGainPercent(currentPrice: number, avgPrice: number): number {
    if (avgPrice === 0) return 0;
    return ((currentPrice - avgPrice) / avgPrice) * 100;
  }

  calculateGainDollars(
    currentPrice: number,
    avgPrice: number,
    quantity: number,
  ): number {
    return (currentPrice - avgPrice) * quantity;
  }

  calculateConsensus(strategies: Strategy[]): ConsensusType {
    if (!strategies || strategies.length === 0) return 'Hold';
    let buy = 0;
    let sell = 0;
    let hold = 0;
    for (const s of strategies) {
      if (s.recommendation === 'BUY') buy++;
      else if (s.recommendation === 'SELL') sell++;
      else hold++;
    }
    if (buy >= 4) return 'Strong Buy';
    if (buy >= 3) return 'Buy';
    if (sell >= 4) return 'Strong Sell';
    if (sell >= 3) return 'Sell';
    return 'Hold';
  }

  enrichPositions(
    trades: OpenTradeWithRecommendations[],
  ): PositionWithGain[] {
    return trades.map((t) => ({
      ...t,
      gainPercent: this.calculateGainPercent(t.current_price, t.prix_moyen),
      gainDollars: this.calculateGainDollars(
        t.current_price,
        t.prix_moyen,
        t.quantite_totale,
      ),
      consensus: this.calculateConsensus(t.strategies),
    }));
  }

  totalGain(closedTrades: ClosedTrade[]): number {
    return closedTrades.reduce((sum, t) => sum + t.gain_dollars, 0);
  }

  avgHoldingTime(closedTrades: ClosedTrade[]): number {
    if (closedTrades.length === 0) return 0;
    const total = closedTrades.reduce((sum, t) => sum + t.temps_jours, 0);
    return Math.round(total / closedTrades.length);
  }

  winLossRatio(closedTrades: ClosedTrade[]): { wins: number; losses: number; ratio: number } {
    const wins = closedTrades.filter((t) => t.gain_dollars > 0).length;
    const losses = closedTrades.filter((t) => t.gain_dollars < 0).length;
    return {
      wins,
      losses,
      ratio: losses === 0 ? wins : +(wins / losses).toFixed(2),
    };
  }

  totalProfits(closedTrades: ClosedTrade[]): number {
    return closedTrades
      .filter((t) => t.gain_dollars > 0)
      .reduce((sum, t) => sum + t.gain_dollars, 0);
  }

  totalLosses(closedTrades: ClosedTrade[]): number {
    return closedTrades
      .filter((t) => t.gain_dollars < 0)
      .reduce((sum, t) => sum + t.gain_dollars, 0);
  }

  totalDeposited(history: WalletHistoryItem[]): number {
    return history
      .filter((t) => t.action === 'ajout')
      .reduce((sum, t) => sum + t.amount, 0);
  }

  totalWithdrawn(history: WalletHistoryItem[]): number {
    return history
      .filter((t) => t.action === 'retrait')
      .reduce((sum, t) => sum + t.amount, 0);
  }

  roi(gains: number, losses: number, totalDeposited: number): number {
    if (totalDeposited === 0) return 0;
    return ((gains + losses) / totalDeposited) * 100;
  }
}
