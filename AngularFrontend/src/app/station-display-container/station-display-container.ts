import { Component, inject, OnInit } from '@angular/core';
import { StationService } from '../station-service';
import { StationDisplaySummary } from './station-display-summary/station-display-summary';

@Component({
  selector: 'app-station-display-container',
  imports: [StationDisplaySummary],
  templateUrl: './station-display-container.html',
  styleUrl: './station-display-container.css',
})
export class StationDisplayContainer implements OnInit{
  _StationService = inject(StationService);
  ngOnInit(): void {
    this._StationService.reload_stations();
  }
}
