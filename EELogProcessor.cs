using System;
using System.Collections.Generic;
using System.IO.MemoryMappedFiles;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices.ComTypes;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Text.RegularExpressions;

namespace SleepFrame
{
    public  class EELogEvent
    {
        private string _category;
        private Dictionary<string, Action<Match>> _patterns;


        public EELogEvent(string category, Dictionary<string, Action<Match>> patterns)
        {
            Category = category;
            Patterns = patterns;
        }

        public string Category
        {
            get { return _category; }
            set { _category = value; }
        }


        public Dictionary<string, Action<Match>> Patterns
        {
            get { return _patterns; }
            set { _patterns = value; }
        }

    }
    public static class EELogProcessor
    {
        private static bool _isRunning = false;
        private static List<EELogEvent> _events = new List<EELogEvent>();



        public static void Start()
        {
            _isRunning = true;
            new Thread(() =>
            {
                while (_isRunning)
                {
                    try
                    {
                        if (Helper.lastWFProcessID == -1)
                            continue;
                        string eeLogPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), @"Warframe\EE.log");
                        var fs = new FileStream(eeLogPath, FileMode.Open, FileAccess.Read, FileShare.ReadWrite);
                        using (var sr = new StreamReader(fs))
                        {
                            var lastLine = "";
                            while (Helper.lastWFProcessID != -1)
                            {
                                lastLine = sr.ReadLine();
                                if (lastLine != null)
                                {
                                    ProcessLine(lastLine);
                                }
                            }
                        }
                    }
                    catch
                    {
                    }
                    finally
                    {
                        Thread.Sleep(10000);
                    }
                }
            })
            {
                IsBackground = true
            }.Start();
        }

        public static void Stop()
        {
            _isRunning = false;
        }

        public static void ProcessLine(string line)
        {
            // Check if any of the events match the line
            Dictionary<string, Action<Match>> patterns = _events.SelectMany(x => x.Patterns).ToDictionary(x => x.Key, x => x.Value);
            foreach (KeyValuePair<string, Action<Match>> pattern in patterns)
            {
                Match match = Regex.Match(line, pattern.Key);
                if (match.Success)
                {
                    pattern.Value(match);
                }
            }
        }


        public static void AddEvent(EELogEvent e)
        {
            _events.Add(e);
        }

        public static void RemoveEventByCategory(string category)
        {
            _events.RemoveAll(x => x.Category == category);
        }

        public static void RemoveEvent(EELogEvent e)
        {
            _events.Remove(e);
        }

    }




}
