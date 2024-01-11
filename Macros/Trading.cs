using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace SleepFrame.Macros
{
    public class Trading : MacroBase
    {
        public Trading() : base(new TimeSpan(0, 0, 20))
        {

        }

        public override void Run()
        {
            //System.Threading.Thread.Sleep(Helper.GetRandomDelay(50, 100));
            //Helper.Ctrl_V("WTB Any Rivens for [Glaive]450 [torid][Ocucor][Ceramic dagger]350 [Latron]200 [skana][dual ichor][Dual toxocyst][burston]100 [verglas][laetum][magistar]75 [hate][ogris]50");
            //Helper.Ctrl_V("Hey");
            System.Threading.Thread.Sleep(Helper.GetRandomDelay(1000, 1500));
            Console.WriteLine("{ENTER}");
            SendKeys.SendWait("{ENTER}");
            //System.Threading.Thread.Sleep(Helper.GetRandomDelay(50, 100));
            //SendKeys.SendWait("{BACKSPACE}");
            //System.Threading.Thread.Sleep(Helper.GetRandomDelay(50, 100));
            //SendKeys.SendWait("t");
            //this.Stop();
        }
    }
}
