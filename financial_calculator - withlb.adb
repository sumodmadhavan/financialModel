with Ada.Text_IO;          use Ada.Text_IO;
with Ada.Float_Text_IO;    use Ada.Float_Text_IO;
with Ada.Numerics.Generic_Elementary_Functions;
with Ada.Calendar;         use Ada.Calendar;
with Ada.Calendar.Formatting; use Ada.Calendar.Formatting;
with Ada.Exceptions;       use Ada.Exceptions;

procedure Financial_Calculator is
   -- with lower bound and upper bound
   package Float_Functions is new Ada.Numerics.Generic_Elementary_Functions (Float);
   use Float_Functions;

   type Financial_Params is record
      Num_Years       : Integer;
      Au_Hours        : Float;
      Initial_TSN     : Float;
      Rate_Escalation : Float;
      AIC             : Float;
      HSI_TSN         : Float;
      Overhaul_TSN    : Float;
      HSI_Cost        : Float;
      Overhaul_Cost   : Float;
   end record;

   function Calculate_Financials (Rate : Float; Params : Financial_Params) return Float is
      Cumulative_Profit : Float := 0.0;
      TSN               : Float;
      Escalated_Rate    : Float;
      Engine_Revenue    : Float;
      Total_Revenue     : Float;
      HSI_Cost          : Float;
      Overhaul_Cost     : Float;
   begin
      for Year in 1 .. Params.Num_Years loop
         TSN := Params.Initial_TSN + Params.Au_Hours * Float(Year);
         Escalated_Rate := Rate * (1.0 + Params.Rate_Escalation / 100.0)**Float(Year - 1);

         Engine_Revenue := Params.Au_Hours * Escalated_Rate;
         Total_Revenue := Engine_Revenue * (1.0 + Params.AIC / 100.0);

         if TSN >= Params.HSI_TSN and then (Year = 1 or TSN - Params.Au_Hours < Params.HSI_TSN) then
            HSI_Cost := Params.HSI_Cost;
         else
            HSI_Cost := 0.0;
         end if;

         if TSN >= Params.Overhaul_TSN and then (Year = 1 or TSN - Params.Au_Hours < Params.Overhaul_TSN) then
            Overhaul_Cost := Params.Overhaul_Cost;
         else
            Overhaul_Cost := 0.0;
         end if;

         Cumulative_Profit := Cumulative_Profit + Total_Revenue - (HSI_Cost + Overhaul_Cost);
      end loop;

      return Cumulative_Profit;
   end Calculate_Financials;

   function Objective_Function (Rate : Float; Params : Financial_Params; Target_Profit : Float) return Float is
   begin
      return Calculate_Financials(Rate, Params) - Target_Profit;
   end Objective_Function;

   function Bisection_Method (A, B : in out Float;
                              Tol : Float;
                              Max_Iter : Integer;
                              Params : Financial_Params;
                              Target_Profit : Float) return Float
   is
      FA, FB, FC, C : Float;
   begin
      FA := Objective_Function(A, Params, Target_Profit);
      FB := Objective_Function(B, Params, Target_Profit);

      if FA * FB > 0.0 then
         raise Constraint_Error with "Error: Function values at endpoints have same sign.";
      end if;

      for I in 1 .. Max_Iter loop
         C := (A + B) / 2.0;
         FC := Objective_Function(C, Params, Target_Profit);

         Put_Line("Iteration" & Integer'Image(I) & ": Rate =" & Float'Image(C) & ", Profit Difference =" & Float'Image(FC));

         if abs(FC) < Tol then
            Put_Line("Converged: Found solution within tolerance");
            return C;
         end if;

         if FA * FC < 0.0 then
            B := C;
            FB := FC;
         else
            A := C;
            FA := FC;
         end if;
      end loop;

      raise Constraint_Error with "Bisection method did not converge within" & Integer'Image(Max_Iter) & " iterations";
   end Bisection_Method;

begin
   declare
      Params : Financial_Params := (
         Num_Years       => 10,
         Au_Hours        => 450.0,
         Initial_TSN     => 100.0,
         Rate_Escalation => 5.0,
         AIC             => 10.0,
         HSI_TSN         => 1000.0,
         Overhaul_TSN    => 3000.0,
         HSI_Cost        => 50000.0,
         Overhaul_Cost   => 100000.0
      );
      Initial_Rate : constant Float := 320.0;
      Target_Profit : constant Float := 3_000_000.0;
      Optimal_Rate : Float;
      Initial_Cumulative_Profit, Final_Cumulative_Profit : Float;
      Start_Time, End_Time : Time;
      Elapsed_Time : Duration;
      Lower_Bound, Upper_Bound : Float;
   begin
      Start_Time := Clock;

      Initial_Cumulative_Profit := Calculate_Financials(Initial_Rate, Params);
      Put("Initial Warranty Rate: ");
      Put(Initial_Rate, Fore => 1, Aft => 2, Exp => 0);
      New_Line;
      Put("Initial Cumulative Profit: ");
      Put(Initial_Cumulative_Profit, Fore => 1, Aft => 2, Exp => 0);
      New_Line;

      -- Set initial bounds
      Lower_Bound := 100.0;  -- Adjust these values based on your problem domain
      Upper_Bound := 1000.0;

      -- Check if bounds are appropriate
      if Objective_Function(Lower_Bound, Params, Target_Profit) * Objective_Function(Upper_Bound, Params, Target_Profit) >= 0.0 then
         Put_Line("Error: Inappropriate bounds for bisection method.");
         Put_Line("Try adjusting Lower_Bound and Upper_Bound to ensure they bracket the solution.");
         return;
      end if;

      Optimal_Rate := Bisection_Method(Lower_Bound, Upper_Bound, 1.0E-8, 100, Params, Target_Profit);

      Put("Optimal Warranty Rate to achieve ");
      Put(Target_Profit, Fore => 1, Aft => 2, Exp => 0);
      Put(" profit: ");
      Put(Optimal_Rate, Fore => 1, Aft => 7, Exp => 0);
      New_Line;

      Final_Cumulative_Profit := Calculate_Financials(Optimal_Rate, Params);
      Put("Final Cumulative Profit: ");
      Put(Final_Cumulative_Profit, Fore => 1, Aft => 2, Exp => 0);
      New_Line;

      End_Time := Clock;
      Elapsed_Time := End_Time - Start_Time;

      Put("Execution time: ");
      Put(Float(Elapsed_Time), Fore => 1, Aft => 6, Exp => 0);
      Put_Line(" seconds");

   exception
      when E : others => Put_Line("Error: " & Exception_Information(E));
   end;
end Financial_Calculator;
