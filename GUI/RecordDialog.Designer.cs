namespace SleepFrame
{
    partial class RecordDialog
    {
        /// <summary>
        /// Required designer variable.
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        /// Clean up any resources being used.
        /// </summary>
        /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Windows Form Designer generated code

        /// <summary>
        /// Required method for Designer support - do not modify
        /// the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            this.components = new System.ComponentModel.Container();
            this.lblRecord = new System.Windows.Forms.Label();
            this.label2 = new System.Windows.Forms.Label();
            this.label3 = new System.Windows.Forms.Label();
            this.label4 = new System.Windows.Forms.Label();
            this.txtY = new System.Windows.Forms.TextBox();
            this.txtX = new System.Windows.Forms.TextBox();
            this.txtRecotdsCount = new System.Windows.Forms.TextBox();
            this.timer1 = new System.Windows.Forms.Timer(this.components);
            this._btnCopyList = new System.Windows.Forms.Button();
            this.SuspendLayout();
            // 
            // lblRecord
            // 
            this.lblRecord.BorderStyle = System.Windows.Forms.BorderStyle.FixedSingle;
            this.lblRecord.Cursor = System.Windows.Forms.Cursors.Hand;
            this.lblRecord.Font = new System.Drawing.Font("Microsoft Sans Serif", 20F, System.Drawing.FontStyle.Bold, System.Drawing.GraphicsUnit.Point, ((byte)(0)));
            this.lblRecord.Location = new System.Drawing.Point(4, 5);
            this.lblRecord.Name = "lblRecord";
            this.lblRecord.Size = new System.Drawing.Size(100, 35);
            this.lblRecord.TabIndex = 0;
            this.lblRecord.Text = "Record";
            this.lblRecord.TextAlign = System.Drawing.ContentAlignment.MiddleCenter;
            this.lblRecord.MouseCaptureChanged += new System.EventHandler(this.lblRecord_MouseCaptureChanged);
            this.lblRecord.MouseDown += new System.Windows.Forms.MouseEventHandler(this.lblRecord_MouseDown);
            // 
            // label2
            // 
            this.label2.AutoSize = true;
            this.label2.Font = new System.Drawing.Font("Microsoft Sans Serif", 8.25F, System.Drawing.FontStyle.Bold, System.Drawing.GraphicsUnit.Point, ((byte)(0)));
            this.label2.Location = new System.Drawing.Point(110, 18);
            this.label2.Name = "label2";
            this.label2.Size = new System.Drawing.Size(19, 13);
            this.label2.TabIndex = 1;
            this.label2.Text = "X:";
            // 
            // label3
            // 
            this.label3.AutoSize = true;
            this.label3.Font = new System.Drawing.Font("Microsoft Sans Serif", 8.25F, System.Drawing.FontStyle.Bold, System.Drawing.GraphicsUnit.Point, ((byte)(0)));
            this.label3.Location = new System.Drawing.Point(176, 18);
            this.label3.Name = "label3";
            this.label3.Size = new System.Drawing.Size(19, 13);
            this.label3.TabIndex = 2;
            this.label3.Text = "Y:";
            // 
            // label4
            // 
            this.label4.AutoSize = true;
            this.label4.Font = new System.Drawing.Font("Microsoft Sans Serif", 8.25F, System.Drawing.FontStyle.Bold, System.Drawing.GraphicsUnit.Point, ((byte)(0)));
            this.label4.Location = new System.Drawing.Point(248, 18);
            this.label4.Name = "label4";
            this.label4.Size = new System.Drawing.Size(58, 13);
            this.label4.TabIndex = 3;
            this.label4.Text = "Recotds:";
            // 
            // txtY
            // 
            this.txtY.BackColor = System.Drawing.SystemColors.Control;
            this.txtY.Location = new System.Drawing.Point(201, 15);
            this.txtY.Name = "txtY";
            this.txtY.ReadOnly = true;
            this.txtY.Size = new System.Drawing.Size(41, 20);
            this.txtY.TabIndex = 4;
            this.txtY.Text = "0";
            // 
            // txtX
            // 
            this.txtX.BackColor = System.Drawing.SystemColors.Control;
            this.txtX.Location = new System.Drawing.Point(129, 15);
            this.txtX.Name = "txtX";
            this.txtX.ReadOnly = true;
            this.txtX.Size = new System.Drawing.Size(41, 20);
            this.txtX.TabIndex = 5;
            this.txtX.Text = "0";
            // 
            // txtRecotdsCount
            // 
            this.txtRecotdsCount.BackColor = System.Drawing.SystemColors.Control;
            this.txtRecotdsCount.Location = new System.Drawing.Point(312, 15);
            this.txtRecotdsCount.Name = "txtRecotdsCount";
            this.txtRecotdsCount.ReadOnly = true;
            this.txtRecotdsCount.Size = new System.Drawing.Size(77, 20);
            this.txtRecotdsCount.TabIndex = 4;
            this.txtRecotdsCount.Text = "0";
            // 
            // timer1
            // 
            this.timer1.Interval = 10;
            this.timer1.Tick += new System.EventHandler(this.timer1_Tick);
            // 
            // _btnCopyList
            // 
            this._btnCopyList.Location = new System.Drawing.Point(395, 12);
            this._btnCopyList.Name = "_btnCopyList";
            this._btnCopyList.Size = new System.Drawing.Size(75, 23);
            this._btnCopyList.TabIndex = 6;
            this._btnCopyList.Text = "Copy List";
            this._btnCopyList.UseVisualStyleBackColor = true;
            this._btnCopyList.Click += new System.EventHandler(this.BtnCopyList_Click);
            // 
            // RecordDialog
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 13F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.ClientSize = new System.Drawing.Size(473, 47);
            this.ControlBox = false;
            this.Controls.Add(this._btnCopyList);
            this.Controls.Add(this.txtX);
            this.Controls.Add(this.txtRecotdsCount);
            this.Controls.Add(this.txtY);
            this.Controls.Add(this.label4);
            this.Controls.Add(this.label3);
            this.Controls.Add(this.label2);
            this.Controls.Add(this.lblRecord);
            this.FormBorderStyle = System.Windows.Forms.FormBorderStyle.FixedSingle;
            this.MaximizeBox = false;
            this.MinimizeBox = false;
            this.Name = "RecordDialog";
            this.ShowInTaskbar = false;
            this.StartPosition = System.Windows.Forms.FormStartPosition.Manual;
            this.TopMost = true;
            this.MouseDown += new System.Windows.Forms.MouseEventHandler(this.RecordDialog_MouseDown);
            this.ResumeLayout(false);
            this.PerformLayout();

        }

        #endregion

        private System.Windows.Forms.Label lblRecord;
        private System.Windows.Forms.Label label2;
        private System.Windows.Forms.Label label3;
        private System.Windows.Forms.Label label4;
        private System.Windows.Forms.TextBox txtY;
        private System.Windows.Forms.TextBox txtX;
        private System.Windows.Forms.TextBox txtRecotdsCount;
        private System.Windows.Forms.Timer timer1;
        private System.Windows.Forms.Button _btnCopyList;
    }
}