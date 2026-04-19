import { ComponentFixture, TestBed } from '@angular/core/testing';

import { StationDisplayThermalImaging } from './station-display-thermal-imaging';

describe('StationDisplayThermalImaging', () => {
  let component: StationDisplayThermalImaging;
  let fixture: ComponentFixture<StationDisplayThermalImaging>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [StationDisplayThermalImaging],
    }).compileComponents();

    fixture = TestBed.createComponent(StationDisplayThermalImaging);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
