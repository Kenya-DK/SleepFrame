using System;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
namespace SleepFrame
{

    public static class Helper
    {
        public static int lastWFProcessID = 0;
        private enum ShowWindowCommands : int
        {
            Hide = 0,
            Normal = 1,
            Minimized = 2,
            Maximized = 3,
        }

        [DllImport("user32.dll")]
        private static extern int SetForegroundWindow(IntPtr hwnd);
        [DllImport("user32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        public static extern bool IsIconic(IntPtr hWnd);

        [DllImport("user32.dll")]
        private static extern bool ShowWindowAsync(IntPtr hWnd, ShowWindowCommands cmdShow);

        [DllImport("user32.dll")]
        public static extern IntPtr GetWindowThreadProcessId(IntPtr hWnd, out uint ProcessId);

        [DllImport("user32.dll")]
        private static extern IntPtr GetForegroundWindow();
        [DllImport("user32.dll")]
        public static extern bool GetWindowRect(IntPtr hwnd, ref Rect rectangle);

        /// Return Type: BOOL->int
        ///fBlockIt: BOOL->int
        [System.Runtime.InteropServices.DllImportAttribute("user32.dll", EntryPoint = "BlockInput")]
        [return: System.Runtime.InteropServices.MarshalAsAttribute(System.Runtime.InteropServices.UnmanagedType.Bool)]
        public static extern bool BlockInput([System.Runtime.InteropServices.MarshalAsAttribute(System.Runtime.InteropServices.UnmanagedType.Bool)] bool fBlockIt);

        public struct Rect
        {
            public int Left { get; set; }
            public int Top { get; set; }
            public int Right { get; set; }
            public int Bottom { get; set; }
            // Method to get the center of the rectangle
            public (int X, int Y) GetCenter()
            {
                int centerX = (Left + Right) / 2;
                int centerY = (Top + Bottom) / 2;

                return (centerX, centerY);
            }
        }


        static Helper()
        {
            new Thread(() =>
            {
                while (true)
                {
                    try
                    {
                        while (!Helper.CheckIsWarframeIsOpen())
                            Thread.Sleep(5000);
                        Thread.Sleep(1500);
                        while (Helper.CheckIsWarframeIsOpen())
                            Thread.Sleep(5000);
                    }
                    catch (Exception ex)
                    {
                        Thread.Sleep(15000);
                    }
                }
            })
            {
                IsBackground = true
            }.Start();
        }


        public static Process GetWarframeProcess()
        {
            return Process.GetProcessesByName("Warframe.x64").FirstOrDefault();
        }

        public static bool CheckIsWarframeIsOpen()
        {
            Process process = GetWarframeProcess();
            lastWFProcessID = process == null ? -1 : process.Id;
            return lastWFProcessID < 0;
        }

        public static Process GetActiveProcess()
        {
            IntPtr hwnd = GetForegroundWindow();
            GetWindowThreadProcessId(hwnd, out uint pid);
            return Process.GetProcessById((int)pid);
        }

        public static void SetProcessToForeground(Process process)
        {
            if (process == null)
                return;
            // check if the process is running
            if (process != null)
            {
                if (IsIconic(process.MainWindowHandle))
                    ShowWindowAsync(process.MainWindowHandle, ShowWindowCommands.Normal);

                // set user the focus to the window
                SetForegroundWindow(process.MainWindowHandle);
            }
        }

        public static int GetRandomDelay(int min, int max)
        {
            Random random = new Random();
            return random.Next(min, max);
        }
        public static class Clipboard
        {
            public static void SetText(string p_Text)
            {
                Thread STAThread = new Thread(
                    delegate ()
                    {
                        // Use a fully qualified name for Clipboard otherwise it
                        // will end up calling itself.
                        System.Windows.Forms.Clipboard.SetText(p_Text);
                    });
                STAThread.SetApartmentState(ApartmentState.STA);
                STAThread.Start();
                STAThread.Join();
            }

            public static string GetText()
            {
                string ReturnValue = string.Empty;
                Thread STAThread = new Thread(
                    delegate ()
                    {
                        // Use a fully qualified name for Clipboard otherwise it
                        // will end up calling itself.
                        ReturnValue = System.Windows.Forms.Clipboard.GetText();
                    });
                STAThread.SetApartmentState(ApartmentState.STA);
                STAThread.Start();
                STAThread.Join();

                return ReturnValue;
            }
        }
    }
}
