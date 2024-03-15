using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace SleepFrame.Macros
{
    public class TheIndex : MacroBase
    {
        public TheIndex() : base("TheIndex", new TimeSpan(0, 0, 10))
        {

        }
        public override void Run()
        {

        }
        public override UserControl GetView()
        {
            return new Views.TheIndexUserControl(this);
        }
    }
}
