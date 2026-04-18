import { ComponentFixture, TestBed } from '@angular/core/testing';

import { StationDisplayContainer } from './station-display-container';

describe('StationDisplayContainer', () => {
  let component: StationDisplayContainer;
  let fixture: ComponentFixture<StationDisplayContainer>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [StationDisplayContainer],
    }).compileComponents();

    fixture = TestBed.createComponent(StationDisplayContainer);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
