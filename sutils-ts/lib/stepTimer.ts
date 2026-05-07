import { ObjectStore } from "./externalStore";

export class StepTimer {
    State;
    Interval = NaN;
    Step = 100;
    OnTick = (progress: number) => progress;
    OnFinish = () => { };
    constructor(finish: number, { Step = 100, OnFinish, OnTick }: Partial<StepTimer> = {}) {
        this.State = new ObjectStore({ progress: 0, finish });
        this.Step = Step;
        this.OnFinish = OnFinish ?? this.OnFinish;
        this.OnTick = OnTick ?? this.OnTick;
    }

    Reset = (finish: number) => this.Start() && this.State.update({ progress: 0, finish });
    Stop = () => {
        if (Number.isNaN(this.Interval)) return;
        clearInterval(this.Interval);
        this.Interval = NaN;
    };
    Start = () => {
        this.Stop();
        this.Interval = setInterval(() => this.State.mutate(({ progress, finish }) => {
            progress = this.OnTick(progress + this.Step);
            if (progress >= finish) {
                clearInterval(this.Interval);
                this.OnFinish();
            }
            return { progress };
        }), this.Step
        );
        return true;
    };
}
