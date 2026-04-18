import { ComponentFixture, TestBed } from '@angular/core/testing';

import { StationDisplaySummary } from './station-display-summary';

describe('StationDisplaySummary', () => {
  let component: StationDisplaySummary;
  let fixture: ComponentFixture<StationDisplaySummary>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [StationDisplaySummary],
    }).compileComponents();

    fixture = TestBed.createComponent(StationDisplaySummary);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
