import { Component, input, OnInit, signal } from '@angular/core';
import { Station } from '../../../station-service';
import { MultiSelectModule } from 'primeng/multiselect';
import { FormsModule } from '@angular/forms';
import { DatePickerModule } from 'primeng/datepicker';
import { ChartModule } from 'primeng/chart';
@Component({
  selector: 'app-station-display-data',
  imports: [MultiSelectModule, FormsModule, DatePickerModule, ChartModule],
  templateUrl: './station-display-data.html',
  styleUrl: './station-display-data.css',
})
export class StationDisplayData implements OnInit {
  readonly station = input.required<Station>();
  readonly data_options = [
    { label: "Air Temperature", value: "air_temperature" },
    { label: "Air Humidity", value: "air_humidity" },
    { label: "Co2 concentration (ppm)", value: "co2_concentration" },
    { label: "Barometric Pressure", value: "barometric_pressure" },
    { label: "Ground Temperature", value: "ground_temperature" },
    { label: "Water Temperature", value: "water_temperature" },
    { label: "Evaporation Rate", value: "evaporation_rate" },
  ]
  readonly current_date = new Date();
  selected_data = signal<{ label: string, value: string }[]>([]);
  start_datetime = signal<Date | null>(null);
  end_datetime = signal<Date | null>(new Date());
  data_loaded = signal<boolean>(false);
  chart_data: any;
  chart_options: any;
  ngOnInit(){
    this.init_chart();
  }
  init_chart(){
    this.chart_data = {
      labels: [1,2,3,4,5,6,7],
      datasets: [
        {
          label: "Air Temperature",
          data: [13,14,15,16,20,14,13]
        }
      ]
    }

  } 
}