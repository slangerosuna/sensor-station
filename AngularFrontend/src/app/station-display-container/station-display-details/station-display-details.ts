import { Component, input, signal } from '@angular/core';
import { Station } from '../../station-service';
import { StationDisplayStaticDetails } from '../station-display-tabs/station-display-static-details/station-display-static-details';
import { StationDisplayEditDetails } from '../station-display-tabs/station-display-edit-details/station-display-edit-details';
import { StationDisplayData } from '../station-display-tabs/station-display-data/station-display-data';
import { StationDisplayContainer } from "../station-display-container";
import { StationDisplayThermalImaging } from "../station-display-tabs/station-display-thermal-imaging/station-display-thermal-imaging";

@Component({
  selector: 'app-station-display-details',
  imports: [StationDisplayStaticDetails, StationDisplayEditDetails, StationDisplayData, StationDisplayThermalImaging],
  templateUrl: './station-display-details.html',
  styleUrl: './station-display-details.css',
})
export class StationDisplayDetails {
  station = input.required<Station>();
  selected_tab = signal<"static_details"|"edit_details"|"data"|"thermal_imagery_setup">("static_details");
}
