import { Component, computed, input, signal } from '@angular/core';
import { Station } from '../../station-service';
import { StationDisplayDetails } from '../station-display-details/station-display-details';

@Component({
  selector: 'app-station-display-summary',
  imports: [StationDisplayDetails],
  templateUrl: './station-display-summary.html',
  styleUrl: './station-display-summary.css',
})
export class StationDisplaySummary {
  readonly station = input.required<Station>();
  readonly health_status = computed<"green" | "yellow" | "red">(() => {
    const now = new Date();
    const last_updated = this.station().lastUpdated;
    const diff_minutes = (now.getTime() - last_updated.getTime()) / 1000 / 60;
    if (diff_minutes < 5) {
      return "green";
    } else if (diff_minutes < 15) {
      return "yellow";
    } else {
      return "red";
    }
  });
  readonly expanded = signal(false);
  toggle(): void {
    this.expanded.update((value) => !value);
  }
  onSummaryKeydown(event: KeyboardEvent): void {
    if (event.key !== 'Enter' && event.key !== ' ') {
      return;
    }
    event.preventDefault();
    this.toggle();
  }
}
