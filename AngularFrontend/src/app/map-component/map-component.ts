import { Component, effect, inject, OnInit } from '@angular/core';
import { LeafletDirective, LeafletLayersControlDirective, LeafletBaseLayersDirective } from '@bluehalo/ngx-leaflet';
import * as L from "leaflet";
import { Control, map } from 'leaflet';
import { StationService } from '../station-service';
import { StaticDataEditingService } from '../static-data-editing-service';
@Component({
  selector: 'app-map-component',
  imports: [LeafletDirective, LeafletBaseLayersDirective, LeafletLayersControlDirective],
  templateUrl: './map-component.html',
  styleUrl: './map-component.css',
})
export class MapComponent implements OnInit {
  readonly osm_layer =L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {attribution: '&amp;copy; OpenStreetMap contributors'});
  readonly esri_satellite_layer = L.tileLayer('https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}',{attribution:'Tiles &copy; Esri &mdash; Source: Esri, i-cubed, USDA, USGS, AEX, GeoEye, Getmapping, Aerogrid, IGN, IGP, UPR-EGP, and the GIS User Community'})
  readonly baseLayers = {
      'OpenStreetMaps': this.osm_layer,
      'Satellite Imagery': this.esri_satellite_layer
  };
  readonly layer_control_options: Control.LayersOptions = {
      
  }
  readonly options = {
    layers: [
      this.esri_satellite_layer
    ],
    zoom: 13,
    center: L.latLng([ 37.3597657, -120.4267336])
  }; 
  
  private readonly _station_service = inject(StationService);
  private readonly _static_data_editing_service = inject(StaticDataEditingService);
  
  private map!: L.Map;
  private station_marker_group = L.layerGroup();

  private station_marker_map = new Map<number, L.Marker>();

  onMapReady(map: L.Map) {
    this.map = map;
    this.station_marker_group.addTo(this.map);
  }
  ngOnInit(): void {
    
  }
  constructor(){
    effect(() => {
      const stations = this._station_service.stations();
      if(stations && this.map) {
        this.station_marker_group.clearLayers();
        stations.forEach(station => {
          const marker = L.marker([station.StaticStationData.latitude, station.StaticStationData.longitude],{draggable: true}).addTo(this.station_marker_group)
          this.station_marker_map.set(station.StaticStationData.id, marker);
          marker.dragging?.disable();
        });
      }
    });
    effect(() => {
      const editing_station = this._static_data_editing_service.editing_station();
      if(editing_station && this.map) {
        const marker = this.station_marker_map.get(editing_station.StaticStationData.id);
        if(marker && marker.dragging) {
          marker.on("drag", () => {
            const latlng = marker.getLatLng();
            editing_station.StaticStationData.latitude = latlng.lat;
            editing_station.StaticStationData.longitude = latlng.lng;
          });
          marker.dragging.enable();
        }
      }
    });
  }
}
