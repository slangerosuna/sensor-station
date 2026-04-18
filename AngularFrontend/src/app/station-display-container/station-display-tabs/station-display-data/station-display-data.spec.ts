import { ComponentFixture, TestBed } from '@angular/core/testing';

import { StationDisplayData } from './station-display-data';

describe('StationDisplayData', () => {
  let component: StationDisplayData;
  let fixture: ComponentFixture<StationDisplayData>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [StationDisplayData],
    }).compileComponents();

    fixture = TestBed.createComponent(StationDisplayData);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
