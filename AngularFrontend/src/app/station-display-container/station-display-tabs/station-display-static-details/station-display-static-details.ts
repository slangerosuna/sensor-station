import { Component, input } from '@angular/core';
import { Station } from '../../../station-service';

@Component({
  selector: 'app-station-display-static-details',
  imports: [],
  templateUrl: './station-display-static-details.html',
  styleUrl: './station-display-static-details.css',
})
export class StationDisplayStaticDetails {
  readonly station = input.required<Station>();
}
