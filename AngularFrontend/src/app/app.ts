import { Component, signal } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { MapComponent } from './map-component/map-component';
import { StationDisplayContainer } from "./station-display-container/station-display-container";

@Component({
  selector: 'app-root',
  imports: [RouterOutlet, MapComponent, StationDisplayContainer],
  templateUrl: './app.html',
  styleUrl: './app.css'
})
export class App {
  protected readonly title = signal('AngularFrontend');
}
