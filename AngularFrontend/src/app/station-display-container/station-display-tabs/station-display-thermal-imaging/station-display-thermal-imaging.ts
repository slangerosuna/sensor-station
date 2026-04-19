import { Component, ElementRef, inject, input, OnInit, signal, ViewChild, viewChild } from '@angular/core';
import { Station } from '../../../station-service';
import { HttpClient } from '@angular/common/http';
import { ButtonModule } from 'primeng/button';
import { SelectButtonModule } from 'primeng/selectbutton';
import { FormsModule } from '@angular/forms';
@Component({
  selector: 'app-station-display-thermal-imaging',
  imports: [ButtonModule,SelectButtonModule,FormsModule ],
  templateUrl: './station-display-thermal-imaging.html',
  styleUrl: './station-display-thermal-imaging.css',
})
export class StationDisplayThermalImaging implements OnInit {
  _http = inject(HttpClient);
  
  @ViewChild("svgGrid") svgRef!: ElementRef<SVGElement>;

  station = input.required<Station>();
  pixels = signal<ThermalImagePixel[]>([]);
  
  paintOptions = [
    {label: "None", value: "none"},
    {label: "Ground", value: "ground"},
    {label: "Water", value: "water"},
  ]

  water_ground_selected = signal<"water"|"ground"|"none">("none");
  ngOnInit() {
    this.get_thermal_image();
  }
  get_thermal_image(){
    const pixels: ThermalImagePixel[] = [];
    for(let x = 0; x < 24; x+=1){
        for(let y = 0; y < 32; y+=1){
          pixels.push({
            x: x,
            y: y,
            ground: false,
            water: false,
            temperature: 30,
          });
        }
      }
      this.pixels.set(pixels);
    };
  drag(event: MouseEvent){
    if(event.buttons !== 1) return;
    this.svgRef.nativeElement.getBoundingClientRect();
    const x_in_element = event.clientX - this.svgRef.nativeElement.getBoundingClientRect().left;
    const y_in_element = event.clientY - this.svgRef.nativeElement.getBoundingClientRect().top;
    const x = Math.floor(x_in_element / this.svgRef.nativeElement.getBoundingClientRect().width * 32);
    const y = Math.floor(y_in_element / this.svgRef.nativeElement.getBoundingClientRect().height * 24);
    const pixel = this.pixels()[x * 32 + y];
    this.selectPixel(pixel);
  }
  /*get_thermal_image() {
    const pixels: ThermalImagePixel[] = [];
    this._http.get<number[]>("api/get_most_recent_image").subscribe((data) => {
      for(let x = 0; x < 24; x+=1){
        for(let y = 0; y < 32; y+=1){
          pixels.push({
            x: x,
            y: y,
            ground: false,
            water: false,
            temperature: data[x * 32 + y],
          });
        }
      }
    }); 
    this._http.get<string>("api/station/bitmap?surface=water").subscribe((data) => {
      for(let x = 0; x < 24; x+=1){
        for(let y = 0; y < 32; y+=1){
          pixels[x * 32 + y].water = data[x * 32 + y] === "1";
        }
      }
    });
    this._http.get<string>("api/station/bitmap?surface=ground").subscribe((data) => {
      for(let x = 0; x < 24; x+=1){
        for(let y = 0; y < 32; y+=1){
          pixels[x * 32 + y].ground = data[x * 32 + y] === "1";
        }
      }
    });
    this.pixels.set(pixels);

  }*/
  
  get_overlay_color(pixel: ThermalImagePixel): string {
    if(pixel.water){
      return "#0000FF40";
    }
    else if (pixel.ground){
      return "#8B451340";
    }
    return "transparent";
  }
  selectPixel(pixel: ThermalImagePixel) {
    if(this.water_ground_selected() === "none"){
      pixel.ground = false;
      pixel.water = false;
    }
    else if (this.water_ground_selected() === "ground"){
      pixel.ground = true;
      pixel.water = false;
    }
    else if (this.water_ground_selected() === "water"){
      pixel.ground = false;
      pixel.water = true;
    }
  }
  convert_temperature_to_color(temperature: number): string {
    return 'white'; 
  }
  save_bitmap() {    
    const water_bitmap = this.pixels().map(px => px.water ? "1" : "0").join("");
    const ground_bitmap = this.pixels().map(px => px.ground ? "1" : "0").join("");
    this._http.post("api/station/bitmap?surface=water", {bitmap: water_bitmap}).subscribe();
    this._http.post("api/station/bitmap?surface=ground", {bitmap: ground_bitmap}).subscribe();
  }
}
export type ThermalImagePixel = {
  x: number;
  y: number;
  ground: boolean;
  water: boolean;
  temperature: number;
}
